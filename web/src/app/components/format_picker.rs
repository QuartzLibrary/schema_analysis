use leptos::prelude::*;
use leptos::{ev, html};
use leptos_ext::signal::ReadSignalExt;

use crate::{AppState, Format};

pub fn format_picker(state: AppState) -> impl IntoView {
    let user_format: RwSignal<_> = state.user_format.clone().into();
    let format = state.format();
    html::div().class("sa-format").child((
        html::span().class("sa-format__label").child("Format"),
        html::div().class("sa-format__options").attr("role", "group").attr("aria-label", "Data format").child(
            Format::ALL
                .iter()
                .map(move |&fmt| {
                    let is_active = format.is(fmt);
                    html::button()
                        .class("sa-format__btn")
                        .class(("sa-format__btn--active", is_active))
                        .attr("aria-pressed", is_active.map(ToString::to_string))
                        .on(ev::click, move |_| user_format.set(Some(fmt)))
                        .child(fmt.name())
                })
                .collect::<Vec<_>>(),
        ),
    ))
}
