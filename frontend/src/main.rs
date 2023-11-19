use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::{js_sys::JSON, spawn_local};
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
        Route::Home => html! { <CraftFinder /> },
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

#[derive(Properties, PartialEq, Clone)]
pub struct TableProps {
    pub offset: u64,
    pub data: Option<Vec<Craftsman>>,
    // pub update: Callback<(u64, Vec<Craftsman>)>
}

struct MyTable {
    props: TableProps,
}

impl Component for MyTable {
    type Message = ();
    type Properties = TableProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            props: ctx.props().clone(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <table>
                <thead>
                    <tr>
                        <th>{"Name"}</th>
                        <th>{"Ranking"}</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        for self.props.data.clone().unwrap_or(Vec::new()).iter().map(|item| {
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
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

    fn prepare_state(&self) -> Option<String> {
        None
    }

    fn destroy(&mut self, ctx: &Context<Self>) {}
}

#[function_component(CraftFinder)]
fn craftfinder() -> Html {
    // first get 20 into list and trigger loading more with button

    html! {
        <MyTable offset= {0} data={None} />
    }
    // let data = use_state(|| None);

    // {
    //     let data = data.clone();
    //     use_effect(move || {
    //         if data.is_none() {
    //             spawn_local(async move {
    //                 let result = get_craftsmen("99998".to_owned(), offset).await;
    //                 data.set(Some(result));
    //             });
    //         }

    //         || {}
    //     });
    // }

    // match data.as_ref() {
    //     None => {
    //         html! {
    //             <div>{"No server response"}</div>
    //         }
    //     }
    //     Some(craftsmen) => {
    //         let props = TableProps {
    //             data: None,
    //             offset: craftsmen.len() as u64,
    //         };

    //         html! {
    //             <MyTable offset= {props.offset} data={props.data} />
    //         }
    //     }
    // }
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
