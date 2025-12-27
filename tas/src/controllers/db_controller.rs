use iced::Task;

use crate::app::Message;
use crate::db::Database;
use crate::state::DbAction;

/// Spawn an iced Task that executes a DbAction and reports back as Message::ActionDone.
/// Keep UI/controllers clean: all DbAction execution logic lives here.
pub fn task_execute(db: Database, action: DbAction) -> Task<Message> {
    Task::perform(async move { execute(db, action).await }, Message::ActionDone)
}

/// Execute a DbAction against the Database.
/// IMPORTANT: Minimal, no refactors, no invented behavior. Mirrors the previous match logic.
pub async fn execute(db: Database, action: DbAction) -> Result<(), String> {
    match action {
        DbAction::CreateUniverse(n, d) => db.create_universe(n, d).await.map_err(|e| e.to_string()),
        DbAction::DeleteUniverse(id) => db.delete_universe(id).await.map_err(|e| e.to_string()),
        DbAction::InjectDemoData(id) => db.inject_demo_data(id).await.map_err(|e| e.to_string()),
        DbAction::ResetDemoDataScoped(id, scope) => db
            .reset_demo_data_scoped(id, scope)
            .await
            .map_err(|e| e.to_string()),

        DbAction::SnapshotCreate { universe_id, name } => db
            .snapshot_create(universe_id, name)
            .await
            .map_err(|e| e.to_string()),
        DbAction::SnapshotDelete { snapshot_id } => db
            .snapshot_delete(snapshot_id)
            .await
            .map_err(|e| e.to_string()),
        DbAction::SnapshotRestore { snapshot_id } => db
            .snapshot_restore(snapshot_id)
            .await
            .map_err(|e| e.to_string()),

        DbAction::CreateBoard(n) => db.create_board(n).await.map_err(|e| e.to_string()),
        DbAction::DeleteBoard(id) => db.delete_board(id).await.map_err(|e| e.to_string()),

        DbAction::SaveCreature(c, uid) => db.upsert_creature(c, uid).await.map_err(|e| e.to_string()),
        DbAction::ArchiveCreature(id, st) => db.set_creature_archived(id, st).await.map_err(|e| e.to_string()),
        DbAction::DeleteCreature(id) => db.delete_creature(id).await.map_err(|e| e.to_string()),

        DbAction::SaveLocation(l) => db.upsert_location(l).await.map_err(|e| e.to_string()),
        DbAction::DeleteLocation(id) => db.delete_location(id).await.map_err(|e| e.to_string()),

        DbAction::SaveEvent(e) => db.upsert_timeline_event(e).await.map_err(|e| e.to_string()),
        DbAction::DeleteEvent(id) => db.delete_timeline_event(id).await.map_err(|e| e.to_string()),
        DbAction::SaveEra(e) => db.upsert_timeline_era(e).await.map_err(|e| e.to_string()),
        DbAction::DeleteEra(id) => db.delete_timeline_era(id).await.map_err(|e| e.to_string()),

        DbAction::SaveCard(c) => db.upsert_card(c).await.map_err(|e| e.to_string()),
        DbAction::MoveCard(cid, col, pos) => db.move_card(cid, col, pos).await.map_err(|e| e.to_string()),
        DbAction::RebalanceColumn(col) => db.rebalance_column(col).await.map_err(|e| e.to_string()),
        DbAction::DeleteCard(id) => db.delete_card(id).await.map_err(|e| e.to_string()),

        // --- THE FORGE ACTIONS ---
        DbAction::CreateStory(uid, title) => db.create_story(uid, title).await.map_err(|e| e.to_string()),
        DbAction::UpdateStory(s) => db.upsert_story(s).await.map_err(|e| e.to_string()),
        DbAction::DeleteStory(id) => db.delete_story(id).await.map_err(|e| e.to_string()),
        DbAction::CreateScene(sid, title) => db.create_scene(sid, title).await.map_err(|e| e.to_string()),
        DbAction::UpdateScene(s) => db.upsert_scene(s).await.map_err(|e| e.to_string()),
        DbAction::DeleteScene(id) => db.delete_scene(id).await.map_err(|e| e.to_string()),

        // Keep existing behavior as-is
        DbAction::ReorderScene(_, _) => Ok(()),
    }
}
