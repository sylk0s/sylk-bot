use firestore::{FirestoreDb, FirestoreQueryParams};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub fn config_env_var(name: &str) -> Result<String, String> {
   std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}
/*
#[async_trait]
// Can sync with firebase
pub trait CloudSync where Self: Sized + for<'de> Deserialize<'de> + Serialize + Unique + Send {
    // Save an object [obj] to a specific [collection]
    // TODO: Impl -> Serialize
    async fn clsave<T>(&self, collection: &'static str) -> Result<T, E> {
        // TODO: make this better
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;   

        db.delete_by_id(collection, self.uuid()).await?;
        db.create_obj(collection, self.uuid(), self).await?;

        Ok(())
    }

    // Get all objects from a field
    async fn clget<T>()  ->  Result<Vec<T>, E> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;

        let objects: Vec<T> = db.query_obj(
           FirestoreQueryParams::new(Self::clname().into()),).await?;

        Ok(objects)
    }

    // Get the name associated with a type implemeneting this trait.
    fn clname<T>() -> String;
}

// Ensures the object can provide a unique id
pub trait Unique {
    fn uuid(&self) -> u32;
}
*/
