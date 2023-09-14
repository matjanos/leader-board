use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Class {
    pub start_time: u16,
    pub end_time: u16,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
pub struct UserWorkout {
    pub id: u32,
    pub date: String,
    pub user: User,
    pub classes: Class,
    pub waitlist: u32,
    pub waitlist_min_time: u32,
}

#[derive(Debug, Deserialize)]
pub struct UsersWorkoutsApiResponse {
    pub userWorkouts: Vec<UserWorkout>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: u32,
    pub first_name: String,
    pub image: Option<String>,
}