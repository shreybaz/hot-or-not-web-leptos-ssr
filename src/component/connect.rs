use leptos::*;
use leptos_use::{
    storage::{use_local_storage, StringCodec},
    use_event_listener, use_interval_fn, use_window,
};
use reqwest::Url;

use crate::{
    consts::{self, ACCOUNT_CONNECTED_STORE},
    state::auth::{auth_state, SessionResponse},
};

#[component]
pub fn ConnectLogin() -> impl IntoView {
    let (_, write_account_connected, _) =
        use_local_storage::<bool, StringCodec>(ACCOUNT_CONNECTED_STORE);
    let logging_in = create_rw_signal(false);
    let auth = auth_state().identity;
    create_effect(move |_| {
        if auth.with(|a| a.is_none()) {
            return;
        }
        _ = use_event_listener(use_window(), ev::message, move |msg| {
            if Url::parse(&msg.origin())
                .map(|u| u.origin() != consts::AUTH_URL.origin())
                .unwrap_or_default()
            {
                return;
            }
            let data = msg.data().as_string().unwrap();
            let res: SessionResponse = serde_json::from_str(&data).unwrap();
            let identity = res.delegation_identity;
            auth.set(Some(identity));
            logging_in.set(false);
            write_account_connected.set(true);
        });
    });

    view! {
        <button
            on:click=move |ev| {
                ev.prevent_default();
                let window = use_window();
                let window = window.as_ref().unwrap();
                let target = window
                    .open_with_url_and_target(consts::AUTH_URL.as_str(), "_blank")
                    .transpose()
                    .and_then(|w| w.ok())
                    .unwrap();
                let target_c = target.clone();
                _ = use_interval_fn(
                    move || {
                        if target_c.closed().unwrap_or_default() {
                            logging_in.try_set(false);
                        }
                    },
                    500,
                );
                on_cleanup(move || _ = target.close());
                logging_in.set(true);
            }

            class="font-bold rounded-full bg-orange-600 py-3 w-full text-center text-xl text-white"
            disabled=move || logging_in() || auth.with(|a| a.is_none())
        >
            {move || if logging_in() { "Connecting..." } else { "Social Connect" }}

        </button>
    }
}