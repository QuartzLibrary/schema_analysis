use leptos::prelude::*;
use leptos::{ev, html};

use super::output_view::{OUTPUT_TYPES, OutputType};

pub fn output_picker(output_type: RwSignal<OutputType>) -> impl IntoView {
    html::div().class("sa-output-pick").child((
        html::span().class("sa-output-pick__label").child("Output"),
        html::div().class("sa-output-pick__options").attr("role", "group").attr("aria-label", "Output type").child(
            OUTPUT_TYPES
                .iter()
                .map(|&ot| {
                    html::button()
                        .class("sa-output-pick__btn")
                        .class(("sa-output-pick__btn--active", move || {
                            output_type.get() == ot
                        }))
                        .attr("aria-pressed", move || {
                            if output_type.get() == ot {
                                "true"
                            } else {
                                "false"
                            }
                        })
                        .on(ev::click, move |_| output_type.set(ot))
                        .child(ot.name())
                })
                .collect::<Vec<_>>(),
        ),
    ))
}
