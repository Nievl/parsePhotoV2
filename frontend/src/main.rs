use components::add_link::AddLink;
use yew::prelude::*;
mod components;

#[function_component]
fn App() -> Html {
    html! {
        <div class="container">
            <header>{"Gallery"}</header>
            <div class="body">{"Parse photo "}</div>
            <div class="card-deck mb-3 text-center">
                <AddLink />
            </div>
            // <LinkModal />
            // <TableLinks />
            // <NotificationContainer />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
