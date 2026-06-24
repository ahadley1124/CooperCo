mod api;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let health = use_state(|| String::from("loading..."));
    let customers = use_state(Vec::new);
    let error = use_state(|| None::<String>);

    {
        let health = health.clone();
        let customers = customers.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match api::fetch_health().await {
                    Ok(status) => health.set(status),
                    Err(err) => {
                        error.set(Some(format!("health check failed: {err}")));
                        return;
                    }
                }

                match api::fetch_customers().await {
                    Ok(items) => customers.set(items),
                    Err(err) => error.set(Some(format!("customer fetch failed: {err}"))),
                }
            });

            || ()
        });
    }

    html! {
        <>
            <h1>{ "Cooper & Co." }</h1>
            <p>{ "Yew frontend calling Rocket API endpoints." }</p>

            <section class="card">
                <h2>{ "API Health" }</h2>
                <p>{ format!("Status: {}", (*health).as_str()) }</p>
            </section>

            <section class="card">
                <h2>{ "Sample Customers" }</h2>
                {
                    if let Some(message) = &*error {
                        html! { <p>{ message }</p> }
                    } else if customers.is_empty() {
                        html! { <p>{ "No customers returned yet." }</p> }
                    } else {
                        html! {
                            <ul>
                                { for customers.iter().map(|customer| html! {
                                    <li>
                                        <strong>{ &customer.name }</strong>
                                        {
                                            if let Some(email) = &customer.contact_email {
                                                html! { <span>{ format!(" ({email})") }</span> }
                                            } else {
                                                Html::default()
                                            }
                                        }
                                    </li>
                                }) }
                            </ul>
                        }
                    }
                }
            </section>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
