use dioxus::prelude::*;

use crate::components::users_table::UsersTable;

#[component]
pub fn AdminView() -> Element {
    rsx! {
        main { class: "col-span-12 row-span-24 flex flex-col items-center", UsersTable {} }
    }
}
