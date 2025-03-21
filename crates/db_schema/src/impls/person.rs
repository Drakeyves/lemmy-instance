use crate::{
  diesel::OptionalExtension,
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId},
  schema::{comment, community, instance, local_user, person, person_actions, post},
  source::person::{
    Person,
    PersonFollower,
    PersonFollowerForm,
    PersonInsertForm,
    PersonUpdateForm,
  },
  traits::{ApubActor, Crud, Followable},
  utils::{functions::lower, get_conn, now, uplete, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, not},
  expression::SelectableHelper,
  result::Error,
  CombineDsl,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

impl Crud for Person {
  type InsertForm = PersonInsertForm;
  type UpdateForm = PersonUpdateForm;
  type IdType = PersonId;

  // Override this, so that you don't get back deleted
  async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    person::table
      .filter(person::deleted.eq(false))
      .find(person_id)
      .first(conn)
      .await
  }

  async fn create(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    form: &PersonUpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person::table.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Person {
  /// Update or insert the person.
  ///
  /// This is necessary for federation, because Activitypub doesn't distinguish between these
  /// actions.
  pub async fn upsert(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .on_conflict(person::ap_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete_account(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Person, Error> {
    let conn = &mut get_conn(pool).await?;

    // Set the local user info to none
    diesel::update(local_user::table.filter(local_user::person_id.eq(person_id)))
      .set(local_user::email.eq::<Option<String>>(None))
      .execute(conn)
      .await?;

    diesel::update(person::table.find(person_id))
      .set((
        person::display_name.eq::<Option<String>>(None),
        person::avatar.eq::<Option<String>>(None),
        person::banner.eq::<Option<String>>(None),
        person::bio.eq::<Option<String>>(None),
        person::matrix_user_id.eq::<Option<String>>(None),
        person::deleted.eq(true),
        person::updated.eq(Utc::now()),
      ))
      .get_result::<Self>(conn)
      .await
  }

  /// Lists local community ids for all posts and comments for a given creator.
  pub async fn list_local_community_ids(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> Result<Vec<CommunityId>, Error> {
    let conn = &mut get_conn(pool).await?;
    comment::table
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .filter(community::local.eq(true))
      .filter(not(community::deleted))
      .filter(not(community::removed))
      .filter(comment::creator_id.eq(for_creator_id))
      .select(community::id)
      .union(
        post::table
          .inner_join(community::table)
          .filter(community::local.eq(true))
          .filter(post::creator_id.eq(for_creator_id))
          .select(community::id),
      )
      .load::<CommunityId>(conn)
      .await
  }

  pub async fn check_username_taken(pool: &mut DbPool<'_>, username: &str) -> LemmyResult<()> {
    use diesel::dsl::{exists, select};
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      person::table
        .filter(lower(person::name).eq(username.to_lowercase()))
        .filter(person::local.eq(true)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::UsernameAlreadyExists.into())
  }

  pub fn local_url(name: &str, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/u/{name}"))?.into())
  }
}

impl PersonInsertForm {
  pub fn test_form(instance_id: InstanceId, name: &str) -> Self {
    Self::new(name.to_owned(), "pubkey".to_string(), instance_id)
  }
}

impl ApubActor for Person {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    person::table
      .filter(person::deleted.eq(false))
      .filter(person::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
  }

  async fn read_from_name(
    pool: &mut DbPool<'_>,
    from_name: &str,
    include_deleted: bool,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut q = person::table
      .into_boxed()
      .filter(person::local.eq(true))
      .filter(lower(person::name).eq(from_name.to_lowercase()));
    if !include_deleted {
      q = q.filter(person::deleted.eq(false))
    }
    q.first(conn).await.optional()
  }

  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    person_name: &str,
    for_domain: &str,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    person::table
      .inner_join(instance::table)
      .filter(lower(person::name).eq(person_name.to_lowercase()))
      .filter(lower(instance::domain).eq(for_domain.to_lowercase()))
      .select(person::all_columns)
      .first(conn)
      .await
      .optional()
  }
}

impl Followable for PersonFollower {
  type Form = PersonFollowerForm;
  async fn follow(pool: &mut DbPool<'_>, form: &PersonFollowerForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let form = (form, person_actions::followed.eq(now().nullable()));
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }

  /// Currently no user following
  async fn follow_accepted(_: &mut DbPool<'_>, _: CommunityId, _: PersonId) -> Result<Self, Error> {
    Err(Error::NotFound)
  }

  async fn unfollow(
    pool: &mut DbPool<'_>,
    form: &PersonFollowerForm,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(person_actions::table.find((form.follower_id, form.person_id)))
      .set_null(person_actions::followed)
      .set_null(person_actions::follow_pending)
      .get_result(conn)
      .await
  }
}

impl PersonFollower {
  pub async fn list_followers(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<Vec<Person>, Error> {
    let conn = &mut get_conn(pool).await?;
    person_actions::table
      .filter(person_actions::followed.is_not_null())
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .filter(person_actions::target_id.eq(for_person_id))
      .select(person::all_columns)
      .load(conn)
      .await
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    source::{
      comment::{Comment, CommentInsertForm, CommentLike, CommentLikeForm, CommentUpdateForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonFollower, PersonFollowerForm, PersonInsertForm, PersonUpdateForm},
      post::{Post, PostInsertForm, PostLike, PostLikeForm},
    },
    traits::{Crud, Followable, Likeable},
    utils::{build_db_pool_for_tests, uplete},
  };
  use diesel::result::Error;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "holly");

    let inserted_person = Person::create(pool, &new_person).await?;

    let expected_person = Person {
      id: inserted_person.id,
      name: "holly".into(),
      display_name: None,
      avatar: None,
      banner: None,
      banned: false,
      deleted: false,
      published: inserted_person.published,
      updated: None,
      ap_id: inserted_person.ap_id.clone(),
      bio: None,
      local: true,
      bot_account: false,
      private_key: None,
      public_key: "pubkey".to_owned(),
      last_refreshed_at: inserted_person.published,
      inbox_url: inserted_person.inbox_url.clone(),
      matrix_user_id: None,
      ban_expires: None,
      instance_id: inserted_instance.id,
      post_count: 0,
      post_score: 0,
      comment_count: 0,
      comment_score: 0,
    };

    let read_person = Person::read(pool, inserted_person.id).await?;

    let update_person_form = PersonUpdateForm {
      ap_id: Some(inserted_person.ap_id.clone()),
      ..Default::default()
    };
    let updated_person = Person::update(pool, inserted_person.id, &update_person_form).await?;

    let num_deleted = Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_person, read_person);
    assert_eq!(expected_person, inserted_person);
    assert_eq!(expected_person, updated_person);
    assert_eq!(1, num_deleted);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn follow() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let person_form_1 = PersonInsertForm::test_form(inserted_instance.id, "erich");
    let person_1 = Person::create(pool, &person_form_1).await?;
    let person_form_2 = PersonInsertForm::test_form(inserted_instance.id, "michele");
    let person_2 = Person::create(pool, &person_form_2).await?;

    let follow_form = PersonFollowerForm {
      person_id: person_1.id,
      follower_id: person_2.id,
      pending: false,
    };
    let person_follower = PersonFollower::follow(pool, &follow_form).await?;
    assert_eq!(person_1.id, person_follower.person_id);
    assert_eq!(person_2.id, person_follower.follower_id);
    assert!(!person_follower.pending);

    let followers = PersonFollower::list_followers(pool, person_1.id).await?;
    assert_eq!(vec![person_2], followers);

    let unfollow = PersonFollower::unfollow(pool, &follow_form).await?;
    assert_eq!(uplete::Count::only_deleted(1), unfollow);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_user_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_user_agg");

    let another_inserted_person = Person::create(pool, &another_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_site_agg".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );

    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let post_like = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);
    let _inserted_post_like = PostLike::like(pool, &post_like).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let mut comment_like = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(pool, &comment_like).await?;

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    let child_comment_like = CommentLikeForm {
      comment_id: inserted_child_comment.id,
      person_id: another_inserted_person.id,
      score: 1,
    };

    let _inserted_child_comment_like = CommentLike::like(pool, &child_comment_like).await?;

    let person_aggregates_before_delete = Person::read(pool, inserted_person.id).await?;

    assert_eq!(1, person_aggregates_before_delete.post_count);
    assert_eq!(1, person_aggregates_before_delete.post_score);
    assert_eq!(2, person_aggregates_before_delete.comment_count);
    assert_eq!(2, person_aggregates_before_delete.comment_score);

    // Remove a post like
    PostLike::remove(pool, inserted_person.id, inserted_post.id).await?;
    let after_post_like_remove = Person::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_post_like_remove.post_score);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;
    Comment::update(
      pool,
      inserted_child_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let after_parent_comment_removed = Person::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_parent_comment_removed.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_parent_comment_removed.comment_score);

    // Remove a parent comment (the scores should also be removed)
    Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    let after_parent_comment_delete = Person::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_parent_comment_delete.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_parent_comment_delete.comment_score);

    // Add in the two comments again, then delete the post.
    let new_parent_comment = Comment::create(pool, &comment_form, None).await?;
    let _new_child_comment =
      Comment::create(pool, &child_comment_form, Some(&new_parent_comment.path)).await?;
    comment_like.comment_id = new_parent_comment.id;
    CommentLike::like(pool, &comment_like).await?;
    let after_comment_add = Person::read(pool, inserted_person.id).await?;
    assert_eq!(2, after_comment_add.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(1, after_comment_add.comment_score);

    Post::delete(pool, inserted_post.id).await?;
    let after_post_delete = Person::read(pool, inserted_person.id).await?;
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_post_delete.comment_score);
    assert_eq!(0, after_post_delete.comment_count);
    assert_eq!(0, after_post_delete.post_score);
    assert_eq!(0, after_post_delete.post_count);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);
    Person::delete(pool, another_inserted_person.id).await?;

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    // Should be none found
    let after_delete = Person::read(pool, inserted_person.id).await;
    assert!(after_delete.is_err());

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
