use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::{self, Validate};

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct CreateTodo {
    #[validate(length(
        min = 1,
        max = 100,
        message = "At least 1 character and less than 100 characters."
    ))]
    text: String,
}

#[cfg(test)]
impl CreateTodo {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct UpdateTodo {
    #[validate(length(
        min = 1,
        max = 100,
        message = "At least 1 character and less than 100 characters."
    ))]
    text: Option<String>,
    completed: Option<bool>,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

type TodoDatas = HashMap<i32, Todo>;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoDatas>>,
}

impl TodoRepositoryForMemory {
    pub fn new() -> Self {
        TodoRepositoryForMemory {
            store: Arc::default(),
        }
    }

    // write権限を持ったHashMapをスレッドセーフに取得
    fn write_store_ref(&self) -> RwLockWriteGuard<TodoDatas> {
        self.store.write().unwrap()
    }

    // read権限を持ったHashMapをスレッドセーフに取得
    fn read_store_ref(&self) -> RwLockReadGuard<TodoDatas> {
        self.store.read().unwrap()
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_store_ref(); // スレッドセーフな書き込み権限ありHashMap
        let id = (store.len() + 1) as i32; // HashMapの長さ+1をidにする(i32)
        let todo = Todo::new(id, payload.text.clone()); // Todoインスタンスを新しく作成
        store.insert(id, todo.clone()); // store(HashMap)に追加
        todo // Todoを返すことで、作成されたtodoのidやインスタンスを知れる
    }

    fn find(&self, id: i32) -> Option<Todo> {
        let store = self.read_store_ref(); // read権限のあるstore
        store.get(&id).map(|todo| todo.clone()) // 指定されたidをgetして,そのcloneを返す
    }

    fn all(&self) -> Vec<Todo> {
        let store = self.read_store_ref(); // read権限のあるstore
        Vec::from_iter(store.values().map(|todo| todo.clone())) // storeの全データをクローンしたVector
    }

    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref(); // read権限のあるstore
        let todo = store.get(&id).context(RepositoryError::NotFound(id))?; // idnの値をget. なければNotFoundエラー
        let text = payload.text.unwrap_or(todo.text.clone()); // 引数のtext. なければtodoのtextのclone
        let completed = payload.completed.unwrap_or(todo.completed);
        // 新しいtodoを作成
        let todo = Todo {
            id,
            text,
            completed,
        };
        store.insert(id, todo.clone()); // idの場所へinsert
        Ok(todo) // 成功したらOkで新しいtodoを返す
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref(); // 書き込み権限ありsotre
        store.remove(&id).ok_or(RepositoryError::NotFound(id))?; // idのデータがあればremove
        Ok(()) // 成功すればOkを返す
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn todo_crud_scenario() {
        let text = "todo text".to_string();
        let id = 1;
        let expected = Todo::new(id, text.clone());

        // create
        let repository = TodoRepositoryForMemory::new();
        let todo = repository.create(CreateTodo { text });
        assert_eq!(expected, todo);

        // find
        let todo = repository.find(todo.id).unwrap();
        assert_eq!(expected, todo);

        // all
        let todo = repository.all();
        assert_eq!(vec![expected], todo);

        // update
        let text = "update todo text".to_string();
        let todo = repository
            .update(
                id,
                UpdateTodo {
                    text: Some(text.clone()),
                    completed: Some(true),
                },
            )
            .expect("failed update todo");

        assert_eq!(
            Todo {
                id,
                text,
                completed: true,
            },
            todo
        );

        // delete
        let res = repository.delete(id);
        assert!(res.is_ok());
    }
}
