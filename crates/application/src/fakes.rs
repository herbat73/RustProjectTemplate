//! In-memory implementations of the ports, for testing the use cases without any infrastructure.

use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use domain::{
    error::{RepositoryError, RepositoryResult},
    todo::{TodoItem, TodoList, TodoRepository},
    user::{NewUser, User, UserRepository},
};

use crate::{
    error::{AppError, AppResult},
    ports::{AccessClaims, PasswordHasher, ResetKeyStore, ResetMailer, TokenService},
};

#[derive(Default)]
pub struct FakeUsers {
    users: Mutex<Vec<User>>,
}

impl FakeUsers {
    pub fn with(users: Vec<User>) -> Self {
        Self {
            users: Mutex::new(users),
        }
    }
}

pub fn user(id: i32, uuid: &str, username: &str, is_admin: bool) -> User {
    User {
        id,
        uuid: uuid.into(),
        username: username.into(),
        password_hash: "hashed:secret".into(),
        email: format!("{username}@example.com"),
        is_admin,
    }
}

#[async_trait]
impl UserRepository for FakeUsers {
    async fn find_by_username_or_email(&self, value: &str) -> RepositoryResult<Option<User>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.username == value || u.email == value)
            .cloned())
    }
    async fn find_by_uuid(&self, uuid: &str) -> RepositoryResult<Option<User>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.uuid == uuid)
            .cloned())
    }
    async fn list(&self) -> RepositoryResult<Vec<User>> {
        Ok(self.users.lock().unwrap().clone())
    }
    async fn insert(&self, new: NewUser) -> RepositoryResult<User> {
        let mut users = self.users.lock().unwrap();
        if users
            .iter()
            .any(|u| u.username == new.username || u.email == new.email)
        {
            return Err(RepositoryError::Conflict);
        }
        let user = User {
            id: users.len() as i32 + 1,
            uuid: new.uuid,
            username: new.username,
            password_hash: new.password_hash,
            email: new.email,
            is_admin: false,
        };
        users.push(user.clone());
        Ok(user)
    }
    async fn update_password_hash(&self, id: i32, hash: &str) -> RepositoryResult<()> {
        if let Some(u) = self.users.lock().unwrap().iter_mut().find(|u| u.id == id) {
            u.password_hash = hash.into();
        }
        Ok(())
    }
    async fn delete_by_uuid(&self, uuid: &str) -> RepositoryResult<bool> {
        let mut users = self.users.lock().unwrap();
        let before = users.len();
        users.retain(|u| u.uuid != uuid);
        Ok(users.len() < before)
    }
}

#[derive(Default)]
struct TodoData {
    // (owner user id, list); items live in the list
    lists: Vec<(i32, TodoList)>,
    next_id: i32,
}

#[derive(Default)]
pub struct FakeTodos {
    data: Mutex<TodoData>,
}

#[async_trait]
impl TodoRepository for FakeTodos {
    async fn lists_with_items(&self, user_id: i32) -> RepositoryResult<Vec<TodoList>> {
        let data = self.data.lock().unwrap();
        Ok(data
            .lists
            .iter()
            .filter(|(owner, _)| *owner == user_id)
            .map(|(_, l)| l.clone())
            .collect())
    }
    async fn list_exists(&self, user_id: i32, id: i32) -> RepositoryResult<bool> {
        let data = self.data.lock().unwrap();
        Ok(data.lists.iter().any(|(o, l)| *o == user_id && l.id == id))
    }
    async fn insert_list(&self, user_id: i32, title: &str, colour: &str) -> RepositoryResult<i32> {
        let mut data = self.data.lock().unwrap();
        data.next_id += 1;
        let id = data.next_id;
        data.lists.push((
            user_id,
            TodoList {
                id,
                title: Some(title.into()),
                colour: Some(colour.into()),
                items: vec![],
            },
        ));
        Ok(id)
    }
    async fn update_list(
        &self,
        user_id: i32,
        id: i32,
        title: &str,
        colour: Option<&str>,
    ) -> RepositoryResult<bool> {
        let mut data = self.data.lock().unwrap();
        let Some((_, list)) = data
            .lists
            .iter_mut()
            .find(|(o, l)| *o == user_id && l.id == id)
        else {
            return Ok(false);
        };
        list.title = Some(title.into());
        if let Some(colour) = colour {
            list.colour = Some(colour.into());
        }
        Ok(true)
    }
    async fn delete_list(&self, user_id: i32, id: i32) -> RepositoryResult<bool> {
        let mut data = self.data.lock().unwrap();
        let before = data.lists.len();
        data.lists.retain(|(o, l)| !(*o == user_id && l.id == id));
        Ok(data.lists.len() < before)
    }
    async fn insert_item(&self, list_id: i32, title: &str) -> RepositoryResult<i32> {
        let mut data = self.data.lock().unwrap();
        data.next_id += 1;
        let id = data.next_id;
        if let Some((_, list)) = data.lists.iter_mut().find(|(_, l)| l.id == list_id) {
            list.items.push(TodoItem {
                id,
                list_id,
                title: Some(title.into()),
                done: false,
                priority: 0,
                note: None,
            });
        }
        Ok(id)
    }
    async fn update_item(
        &self,
        user_id: i32,
        id: i32,
        title: &str,
        done: bool,
    ) -> RepositoryResult<bool> {
        let mut data = self.data.lock().unwrap();
        for (owner, list) in data.lists.iter_mut() {
            if *owner != user_id {
                continue;
            }
            if let Some(item) = list.items.iter_mut().find(|i| i.id == id) {
                item.title = Some(title.into());
                item.done = done;
                return Ok(true);
            }
        }
        Ok(false)
    }
    async fn update_item_detail(
        &self,
        user_id: i32,
        id: i32,
        list_id: i32,
        priority: i32,
        note: Option<&str>,
    ) -> RepositoryResult<bool> {
        let mut data = self.data.lock().unwrap();
        let mut found = None;
        for (owner, list) in data.lists.iter_mut() {
            if *owner != user_id {
                continue;
            }
            if let Some(pos) = list.items.iter().position(|i| i.id == id) {
                found = Some(list.items.remove(pos));
                break;
            }
        }
        let Some(mut item) = found else {
            return Ok(false);
        };
        item.list_id = list_id;
        item.priority = priority;
        item.note = note.map(str::to_string);
        if let Some((_, target)) = data.lists.iter_mut().find(|(_, l)| l.id == list_id) {
            target.items.push(item);
        }
        Ok(true)
    }
    async fn delete_item(&self, user_id: i32, id: i32) -> RepositoryResult<bool> {
        let mut data = self.data.lock().unwrap();
        for (owner, list) in data.lists.iter_mut() {
            if *owner != user_id {
                continue;
            }
            if let Some(pos) = list.items.iter().position(|i| i.id == id) {
                list.items.remove(pos);
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub struct FakeHasher;

impl PasswordHasher for FakeHasher {
    fn hash(&self, plain: &str) -> AppResult<String> {
        Ok(format!("hashed:{plain}"))
    }
    fn verify(&self, plain: &str, hash: &str) -> bool {
        hash == format!("hashed:{plain}")
    }
}

/// Tokens are readable strings: `requester:<uuid>` and `access:<uuid>:<admin>`.
pub struct FakeTokens;

impl TokenService for FakeTokens {
    fn issue_requester_token(&self, user_uuid: &str) -> AppResult<String> {
        Ok(format!("requester:{user_uuid}"))
    }
    fn issue_access_token(&self, user_uuid: &str, is_admin: bool) -> AppResult<String> {
        Ok(format!("access:{user_uuid}:{is_admin}"))
    }
    fn verify_requester_token(&self, token: &str) -> AppResult<String> {
        token
            .strip_prefix("requester:")
            .map(str::to_string)
            .ok_or(AppError::InvalidToken)
    }
    fn verify_access_token(&self, token: &str) -> AppResult<AccessClaims> {
        let rest = token
            .strip_prefix("access:")
            .ok_or(AppError::InvalidToken)?;
        let (uuid, admin) = rest.split_once(':').ok_or(AppError::InvalidToken)?;
        Ok(AccessClaims {
            user_uuid: uuid.into(),
            is_admin: admin == "true",
        })
    }
}

#[derive(Default)]
pub struct FakeResetKeys {
    keys: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl ResetKeyStore for FakeResetKeys {
    async fn issue(&self, email: &str) -> String {
        let key = format!("key-{}", self.keys.lock().unwrap().len());
        self.keys.lock().unwrap().insert(key.clone(), email.into());
        key
    }
    async fn is_valid(&self, key: &str, email: &str) -> bool {
        self.keys
            .lock()
            .unwrap()
            .get(key)
            .is_some_and(|e| e == email)
    }
    async fn revoke(&self, key: &str) {
        self.keys.lock().unwrap().remove(key);
    }
}

#[derive(Default)]
pub struct FakeMailer {
    pub sent: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl ResetMailer for FakeMailer {
    async fn send_reset_link(&self, email: &str, reset_key: &str) -> AppResult<()> {
        self.sent
            .lock()
            .unwrap()
            .push((email.into(), reset_key.into()));
        Ok(())
    }
}
