// Copyright 2020 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::collections::HashMap;
use std::sync::Arc;

use common_exception::ErrorCode;
use common_exception::Result;
use common_infallible::RwLock;
use common_metatypes::MetaId;
use common_metatypes::MetaVersion;
use common_planners::CreateDatabasePlan;
use common_planners::DropDatabasePlan;

use crate::catalogs::catalog::Catalog;
use crate::catalogs::CatalogBackend;
use crate::catalogs::Database;
use crate::catalogs::TableFunctionMeta;
use crate::catalogs::TableMeta;
use crate::configs::Config;

// min id for system tables (inclusive)
pub const SYS_TBL_ID_BEGIN: u64 = 1 << 62;
// max id for system tables (exclusive)
pub const SYS_TBL_ID_END: u64 = SYS_TBL_ID_BEGIN + 10000;

// min id for system tables (inclusive)
// max id for local tables is u64:MAX
pub const LOCAL_TBL_ID_BEGIN: u64 = SYS_TBL_ID_END;

// Maintain all the catalog backends of user.
pub struct DatabaseCatalog {
    catalog_backends: RwLock<HashMap<String, Arc<dyn CatalogBackend>>>,
}

impl DatabaseCatalog {
    pub fn try_create_with_config(_conf: Config) -> Result<Self> {
        Ok(DatabaseCatalog {
            catalog_backends: Default::default(),
        })
    }
}

impl Catalog for DatabaseCatalog {
    fn register_db_engine(
        &self,
        engine_type: &str,
        backend: Arc<dyn CatalogBackend>,
    ) -> Result<()> {
        let engine = engine_type.to_lowercase();
        self.catalog_backends.write().insert(engine, backend);
        Ok(())
    }

    fn get_databases(&self) -> Result<Vec<Arc<dyn Database>>> {
        let mut databases = vec![];
        let backends = self.catalog_backends.read();
        for backend in backends.values() {
            databases.extend(backend.get_databases()?)
        }
        Ok(databases)
    }

    fn get_database(&self, db_name: &str) -> Result<Arc<dyn Database>> {
        let backends = self.catalog_backends.read();
        for backend in backends.values() {
            if let Some(db) = backend.get_database(db_name)? {
                return Ok(db);
            }
        }

        // Can't found in all the backend for the db_name.
        Err(ErrorCode::UnknownDatabase(format!(
            "Unknown database {}",
            db_name
        )))
    }

    fn exists_database(&self, db_name: &str) -> Result<bool> {
        let backends = self.catalog_backends.read();
        for backend in backends.values() {
            if backend.exists_database(db_name)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn get_table(&self, db_name: &str, table_name: &str) -> Result<Arc<TableMeta>> {
        let db = self.get_database(db_name)?;
        db.get_table(table_name)
    }

    fn get_table_by_id(
        &self,
        db_name: &str,
        table_id: MetaId,
        table_version: Option<MetaVersion>,
    ) -> Result<Arc<TableMeta>> {
        let db = self.get_database(db_name)?;
        db.get_table_by_id(table_id, table_version)
    }

    fn get_table_function(&self, func_name: &str) -> Result<Arc<TableFunctionMeta>> {
        let databases = self.get_databases()?;
        for database in databases {
            let funcs = database.get_table_functions()?;
            for func in funcs {
                if func.raw().name() == func_name {
                    return Ok(func);
                }
            }
        }
        Err(ErrorCode::UnknownTableFunction(format!(
            "Unknown table function: '{}'",
            func_name
        )))
    }

    fn create_database(&self, plan: CreateDatabasePlan) -> Result<()> {
        let db_name = plan.db.as_str();
        let exists = self.exists_database(db_name)?;
        if exists {
            if plan.if_not_exists {
                return Ok(());
            } else {
                return Err(ErrorCode::UnknownDatabase(format!(
                    "Database: '{}' already exists.",
                    db_name
                )));
            }
        }

        // Get the database backend and create it.
        let engine = plan.engine.clone().to_string();
        if let Some(backend) = self.catalog_backends.read().get(engine.as_str()) {
            backend.create_database(plan)
        } else {
            Err(ErrorCode::UnknownDatabase(format!(
                "Database: unknown engine '{}'.",
                engine
            )))
        }
    }

    fn drop_database(&self, plan: DropDatabasePlan) -> Result<()> {
        let db_name = plan.db.as_str();
        let backends = self.catalog_backends.read();
        for backend in backends.values() {
            if backend.exists_database(db_name)? {
                return backend.drop_database(plan.clone());
            }
        }

        if plan.if_exists {
            Ok(())
        } else {
            Err(ErrorCode::UnknownDatabase(format!(
                "Unknown database: '{}'",
                plan.db
            )))
        }
    }
}
