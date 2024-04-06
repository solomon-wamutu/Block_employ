use std::{collections::BTreeMap, sync::Arc};
use crate::{prelude::W, utils::macros::map};

use chrono::{DateTime, Utc};
use serde::{Deserialize,Serialize};
use surrealdb::{
    sql::{query, thing, Array, Object, Value},
    Datastore,Response,Session,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub_id:Option<String>,
    pub title:String,
    pub completed:bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at:Option<DateTime<Utc>>,
}

impl From <Task> for Value{
    fn from(val: Task) -> Self {
        match val.id{
            Some(v)=> map![
            "id".into()=>v.into(),
            "title".into() => val.title.into(),
            "completed".into() => val.completed.into(),
            ]
            .into(),
            None => map![
                "title".into() =>val.title.into(),
                "completed".into() =>val.completed.into()
            ]
        }
    }
}

impl Createtable for Task{

}
#[derive(Debug, Serialize, Deserialize)]
pub struct RowId{
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AffectedRows{
    pub rows_affected: u64,
}

pub trait Createtable: Into<Value>{
    #[derive(Clone)]
    pub struct DB{
        pub ds:Arc<Datastore>,
        pub sesh: Session,
    }
}
    
impl DB{
    pub async fn execute(
        &self,
        query:&str,
        vars: Option<BTreeMap<String, Values>>,

    ) -> Result<Vec<Response>, create::errors::Error> {
        let res = self.ds.execute(query, &self.sesh,vars,false).await?;
        Ok(res)
    }

    pub async fn add_task(&self,title:String) -> Result<Object,crate::error::Error> {
        let sql = "CREATE task SET title = $title, completed = false, created_at = $created_at";
        let vars: BTreeMap<String, Value> = map!["titlle".into() => Value::Strand(title)];
        let res = self.execute(sql,Some(vars)).await?;
        let first_res = res.into_iter().next().expect("Did not get a response");
        W(first_res.result?.first()).try_into()
    }

    pub async fn get_task(&self, id: String) -> Result<Object, crate::errors::Error>{
        let sql  = "SELECT * FROM $th";
        let tid = format!("{}", id);
        let vars: BTreeMap<String, Value> = map!["th".into() =>thing()];
    }


}