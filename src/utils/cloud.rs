use firestore::{FirestoreDb, FirestoreQueryParams};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub fn config_env_var(name: &str) -> Result<String, String> {
   std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[async_trait]
// Can sync with firebase
pub trait CloudSync where for<'a> Self: Deserialize<'a> + Serialize + Unique + Sync + Send {
    // Save an object [obj] to a specific [collection]
    async fn clsave<T>(&self, collection: &'static str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: make this better
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;   

        db.delete_by_id(collection, self.uuid().to_string()).await?;
        db.create_obj(collection, self.uuid().to_string(), self).await?;

        Ok(())
    }

    // Get all objects from a field
    async fn clget<T: for<'a> Deserialize<'a>>() ->  Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;
        let objects: Vec<T> = db.query_obj(FirestoreQueryParams::new(Self::clname::<T>().into())).await?;
        Ok(objects)
    }

    // Get the name associated with a type implemeneting this trait.
    fn clname<T>() -> &'static str;
}

// Ensures the object can provide a unique id
pub trait Unique {
    fn uuid(&self) -> u32;
}
