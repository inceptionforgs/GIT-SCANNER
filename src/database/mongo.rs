use mongodb::{Client, Database, Collection};
use mongodb::options::{ClientOptions, IndexOptions, CreateIndexOptions};
use mongodb::bson::doc;
use std::sync::Arc;
use tracing::{info, warn};

pub struct MongoDB {
    client: Client,
    database: Database,
}

impl MongoDB {
    pub async fn new(uri: &str, db_name: &str) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        // Parse client options
        let mut client_options = ClientOptions::parse(uri).await?;
        
        // Set connection pool options
        client_options.max_pool_size = Some(50);
        client_options.min_pool_size = Some(5);
        client_options.max_idle_time = Some(std::time::Duration::from_secs(60));
        client_options.server_selection_timeout = Some(std::time::Duration::from_secs(10));
        
        // Create client
        let client = Client::with_options(client_options)?;
        
        // Get database
        let database = client.database(db_name);
        
        info!("✅ MongoDB connected to database: {}", db_name);
        
        let db = Arc::new(Self { client, database });
        
        // Create indexes
        db.create_indexes().await?;
        
        Ok(db)
    }
    
    pub fn get_collection<T>(&self, name: &str) -> Collection<T> {
        self.database.collection::<T>(name)
    }
    
    async fn create_indexes(&self) -> Result<(), Box<dyn std::error::Error>> {
        let wallets: Collection<mongodb::bson::Document> = self.database.collection("monitored_wallets");
        
        // Unique index on address
        let address_index = mongodb::IndexModel::builder()
            .keys(doc! { "address": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name("unique_address".to_string())
                    .build()
            )
            .build();
        
        // Index on status for fast queries
        let status_index = mongodb::IndexModel::builder()
            .keys(doc! { "status": 1 })
            .options(
                IndexOptions::builder()
                    .name("status_index".to_string())
                    .build()
            )
            .build();
        
        // Index on last_checked for monitoring
        let last_checked_index = mongodb::IndexModel::builder()
            .keys(doc! { "last_checked": 1 })
            .options(
                IndexOptions::builder()
                    .name("last_checked_index".to_string())
                    .build()
            )
            .build();
        
        wallets.create_indexes(vec![
            address_index,
            status_index,
            last_checked_index,
        ]).await?;
        
        info!("✅ MongoDB indexes created");
        
        Ok(())
    }
}