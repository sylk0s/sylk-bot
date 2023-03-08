use firestore::{FirestoreDb, FirestoreQueryParams};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn config_env_var(name: &str) -> Result<String, String> {
   std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[async_trait]
// Can sync with firebase
pub trait CloudSync where for<'a> Self: Deserialize<'a> + Serialize + Unique + Sync + Send {

    // Save an object [obj] to a specific collection
    async fn clsave(&self, collection: &'static str) -> Result<(), Error> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;   
        db.delete_by_id(collection, self.uuid().to_string()).await?;
        db.create_obj(collection, self.uuid().to_string(), self).await?;
        Ok(())
    }

    // Remove a specific object
    async fn clrm(&self) -> Result<(), Error> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;   
        db.delete_by_id(Self::clname(), self.uuid().to_string()).await?;
        Ok(())
    }

    /// Get all objects from a collection
    async fn clget() ->  Result<Vec<Self>, Error> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;
        let objects: Vec<Self> = db.query_obj(FirestoreQueryParams::new(Self::clname().into())).await?;
        Ok(objects)
    }

    /// Get all items from the collection as a HashMap
    async fn clhash() -> Result<HashMap<u64, Self>, Error> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;
        let objects: Vec<Self> = db.query_obj(FirestoreQueryParams::new(Self::clname().into())).await?;
        let mut hash = HashMap::new();
        for obj in objects {
            hash.insert(obj.uuid(), obj);
        }
        Ok(hash)
    }

    // Get the name associated with a type implemeneting this trait.
    fn clname() -> &'static str;
}

// Ensures the object can provide a unique id
pub trait Unique {
    fn uuid(&self) -> u64;
}
