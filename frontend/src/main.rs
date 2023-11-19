use std::ops::Deref;

use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::{js_sys::JSON, spawn_local};
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/hello-server")]
    HelloServer,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => {
            let opt: Option<String> = None;
            let custom_form_submit = Callback::from(|data: Vec<Craftsman>| {});
            html! {
            <CraftFinder /> }
        }
        Route::HelloServer => html! { <HelloServer /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[derive(Deserialize, PartialEq, Clone)]
pub struct Craftsman {
    id: i32,
    name: String,
    #[serde(rename = "rankingScore")]
    ranking_score: f64,
    street: String,
    house_number: String,
    distance: f64,
}

#[derive(Deserialize)]
pub struct APIResponse {
    craftsmen: Vec<Craftsman>,
}

struct Data {
    pub offset: u64,
    pub postcode: String,
    pub data: Vec<Craftsman>,

}

impl Data {
    pub fn init() -> Self {
        Self { offset: 0, postcode: "".to_owned(), data: Vec::new() }
    }

    
}
#[function_component(CraftFinder)]
fn craftfinder() -> Html {
    // first get 20 into list and trigger loading more with button

    let state = use_state(|| Data::init());

    let data = state.clone();
    let postcode_changes = Callback::from(move |postcode : String| {
        let data = data.clone();
        spawn_local(async move {
            let mut result = get_craftsmen(postcode.clone(), data.offset).await;
            let (offset,new)  = if postcode == data.postcode {
                let mut vals = data.data.clone();
                vals.append(&mut result);
                (vals.len() as u64, vals)
            }
            else {
                (0, result)
            };

            data.set(Data { offset, postcode, data: new });
        });
    });

    let form_onsubmit = Callback::from(|_: Vec<Craftsman>| {});

    let data = state.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        let data = data.deref().clone();
        form_onsubmit.emit(data.data.clone());
    });

    let onchange = Callback::from(move |event: Event| {
        let value = event
            .target()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .value();
        postcode_changes.emit(value);
    });

    let data = state.clone();

    html! {
        <><form onsubmit={onsubmit}>
        <input type="text" name={"PLZ"} onchange={onchange} placeholder={""} />
        <button type="submit"> {"Suche"}</button>
        </form>
        <table>
            <thead>
                <tr>
                    <th>{"Name"}</th>
                    <th>{"Ranking"}</th>
                </tr>
            </thead>
            <tbody>
                {
                    for data.data.clone().iter().map(|item| {
                        html! {
                            <tr>
                                <td>{ &item.name }</td>
                                <td>{ &item.ranking_score }</td>
                            </tr>
                        }
                    })
                }
            </tbody>
        </table>
        </>
    }
}

async fn get_craftsmen(postcode: String, offset: u64) -> Vec<Craftsman> {
    // do one request starting from offset
    let resp = Request::get(format!("/craftsmen?postalcode={postcode}&offset={offset}").as_str())
        .send()
        .await
        .unwrap();
    let result = if resp.ok() {
        let response: APIResponse = serde_json::from_value(resp.json().await.unwrap()).unwrap();
        response.craftsmen
    } else {
        Vec::new()
    };
    result
}

#[function_component(HelloServer)]
fn hello_server() -> Html {
    let data = use_state(|| None);

    // Request `/api/hello` once
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/craftsmen?postalcode=10178?offset=20")
                        .send()
                        .await
                        .unwrap();
                    let result: Result<String, String> = {
                        if !resp.ok() {
                            Err(format!(
                                "Error fetching data {} ({})",
                                resp.status(),
                                resp.status_text()
                            ))
                        } else {
                            resp.text().await.map_err(|err| err.to_string())
                        }
                    };
                    data.set(Some(result));
                });
            }

            || {}
        });
    }

    match data.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <div>{"Got server response: "}{data}</div>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
