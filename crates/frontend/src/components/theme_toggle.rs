use dioxus::prelude::*;

const INIT_JS: &str = r#"
let t = null;
try { t = localStorage.getItem('theme'); } catch (_) {}
if (t !== 'dark' && t !== 'light') {
  t = matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}
document.documentElement.setAttribute('data-theme', t);
return t;
"#;

/// Button that switches between dark and light UI mode. The choice is stored in
/// `localStorage` and applied through the `data-theme` attribute on `<html>`.
#[component]
pub fn ThemeToggle() -> Element {
    let mut dark = use_signal(|| false);

    use_hook(|| {
        spawn(async move {
            if let Ok(v) = document::eval(INIT_JS).await {
                dark.set(v.as_str() == Some("dark"));
            }
        });
    });

    let label = if dark() {
        "Switch to light mode"
    } else {
        "Switch to dark mode"
    };

    rsx! {
        button {
            r#type: "button",
            class: "flex items-center justify-center h-full w-12 text-lg cursor-pointer
                text-[var(--secondary-color-4)] hover:bg-[var(--primary-color-5)]
                hover:text-[var(--secondary-color-1)] transition-colors
                focus-visible:outline-none focus-visible:ring-2",
            title: label,
            aria_label: label,
            onclick: move |_| {
                let next = if dark() { "light" } else { "dark" };
                dark.set(next == "dark");
                document::eval(
                    &format!(
                        "document.documentElement.setAttribute('data-theme','{next}');
                         try {{ localStorage.setItem('theme','{next}'); }} catch (_) {{}}"
                    ),
                );
            },
            if dark() {
                "☀"
            } else {
                "☾"
            }
        }
    }
}
