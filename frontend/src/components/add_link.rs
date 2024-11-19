use log::{error, info};
use regex::Regex;
use yew::prelude::*;

#[function_component]
pub fn AddLink() -> Html {
    let path = use_state(|| "".to_string());

    // Функция для обработки отправки
    let on_submit = {
        let path = path.clone();

        Callback::from(move |_| {
            let current_path = path.trim().to_string();

            if current_path.is_empty() {
                error!("Path cannot be empty!");
                return;
            }

            if check_url(&current_path).is_none() {
                error!("Path is incorrect");
                return;
            }

            wasm_bindgen_futures::spawn_local({
                let path = path.clone();
                async move {
                    create_one(&current_path).await;
                    links_get_all().await;
                    path.set("".to_string());
                }
            });
        })
    };

    html! {
        <div class="card card_new">
          <div class="card-body">
            <h5 class="card-title">{"Adding link"}</h5>
            <div class="input-group mb-3">
              <input
                id="link"
                class="form-control"
                value={(*path).clone()}
                oninput={Callback::from(move |e: InputEvent| {
                  if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                      path.set(input.value());
                  }
                })}
              />
              <div class="input-group-append">
                <button class="btn btn-outline-primary" onclick={on_submit}>
                  {"Add"}
                </button>
              </div>
            </div>
          </div>
        </div>
    }
}

fn check_url(url: &str) -> Option<Vec<String>> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Regex::new(r"(http[s]?://[^/\s]+/)(.*)")
            .unwrap()
            .captures(url)
            .map(|caps| {
                vec![
                    caps.get(1).map_or("", |m| m.as_str()).to_string(), // Base URL part
                    caps.get(2).map_or("", |m| m.as_str()).to_string(), // Remaining path part
                ]
            })
    } else {
        None
    }
}

async fn create_one(path: &str) {
    //  заглушка
    info!("Creating a new record with path: {}", path);
}

async fn links_get_all() {
    //  заглушка
    info!("Fetching all links...");
}
