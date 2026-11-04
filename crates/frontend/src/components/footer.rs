use yew::prelude::*;
use crate::components::buttons::external_button::ExternalButton;

#[function_component(Footer)]
pub fn footer() -> Html {

    html! {
        <footer class="flex flex-row justify-self-end w-full bg-slate-100 dark:bg-slate-900 z-50
                bottom-0 left-0 right-0 fixed border-slate-300 dark:border-slate-700 border-t
                justify-evenly justify-items-center items-center shadow-md h-12">
            <ExternalButton label="Yew Docs" destination="https://yew.rs/docs/getting-started/introduction"/>
            <ExternalButton label="Are we web yet?" destination="https://www.arewewebyet.org/"/>
        </footer>
    }
}