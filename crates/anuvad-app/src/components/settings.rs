use leptos::prelude::*;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <div class="card">
            <h2 class="text-lg font-semibold mb-3">"Settings"</h2>
            <div class="space-y-3 text-sm text-gray-600 dark:text-gray-400">
                <p>"Backend: CPU (WASM SIMD)"</p>
                <p>"WebGPU support coming in v2."</p>
            </div>
        </div>
    }
}
