/// Alias definitions for the Blokmap models.
/// Useful for simplifying queries and improving readability, as well as
/// avoiding name conflicts.
use crate::db::schema::{profile, translation};

diesel::alias!(
	translation as description: DescriptionAlias,
	translation as excerpt: ExcerptAlias,
	translation as tag_name: TagNameAlias,
	profile as approver: ApproverAlias,
	profile as rejecter: RejecterAlias,
	profile as creator: CreatorAlias,
	profile as updater: UpdaterAlias,
	profile as confirmer: ConfirmerAlias,
);
