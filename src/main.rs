#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::desktop::tao::dpi::PhysicalPosition;
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use settings::Settings;

mod settings;
mod summarizer;
mod tavily;

// ── カラーパレット ────────────────────────────────────────
const BG: &str = "#0f1117";
const SURFACE: &str = "#1a1d27";
const SURFACE2: &str = "#222536";
const BORDER: &str = "#2e3148";
const TEXT: &str = "#d4d6e4";
const TEXT_DIM: &str = "#7b7e9a";
const ACCENT: &str = "#7c6af7";
const ERROR_BG: &str = "#2a1520";
const ERROR_BORDER: &str = "#6b2030";
const ERROR_TEXT: &str = "#f08090";
const SUCCESS: &str = "#5cc98a";
// ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Idle,
    Loading,
    Done {
        summary: String,
        sources: Vec<(String, String)>,
    },
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Search,
    Settings,
}

fn main() {
    let window = dioxus::desktop::WindowBuilder::new()
        .with_title("WebSearch Launcher")
        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(560.0_f64, 480.0_f64))
        .with_always_on_top(true)
        .with_resizable(false)
        .with_decorations(false);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(window)
                // WebView の初期背景をダーク色に設定して白フラッシュを防ぐ
                .with_custom_head(
                    r#"<style>html,body{background:#0f1117;margin:0;padding:0;}</style>"#
                        .to_string(),
                ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let screen = use_signal(|| Screen::Search);
    let cfg = use_signal(|| settings::load());

    // 初回レンダリング後にウィンドウを画面中央へ移動
    let win = use_window();
    use_effect(move || {
        if let Some(monitor) = win.current_monitor() {
            let m = monitor.size();
            let w = win.outer_size();
            let x = (m.width.saturating_sub(w.width) / 2) as i32;
            let y = (m.height.saturating_sub(w.height) / 2) as i32;
            win.set_outer_position(PhysicalPosition::new(x, y));
        }
    });

    match screen() {
        Screen::Search => rsx! { SearchScreen { screen, cfg } },
        Screen::Settings => rsx! { SettingsScreen { screen, cfg } },
    }
}

#[component]
fn SearchScreen(screen: Signal<Screen>, cfg: Signal<Settings>) -> Element {
    let mut query = use_signal(|| String::new());
    let mut use_news = use_signal(|| false);
    let mut app_state = use_signal(|| AppState::Idle);
    let win = use_window();
    let win_drag = win.clone();
    let win_minimize = win.clone();
    let win_close = win.clone();

    let do_search = move || {
        let q = query.read().trim().to_string();
        if q.is_empty() {
            return;
        }
        let s = cfg.read().clone();
        let api_key = settings::resolve_tavily_key(&s);
        let news = *use_news.read();

        spawn(async move {
            app_state.set(AppState::Loading);

            let results = match tavily::search(&q, &api_key, news).await {
                Ok(r) => r,
                Err(e) => {
                    app_state.set(AppState::Error(e.to_string()));
                    return;
                }
            };

            let sources: Vec<(String, String)> = results
                .iter()
                .map(|r| (r.title.clone(), r.url.clone()))
                .collect();

            match summarizer::summarize(&q, &results, &s.ollama_host, &s.model_name).await {
                Ok(summary) => {
                    app_state.set(AppState::Done { summary, sources });
                }
                Err(e) => {
                    app_state.set(AppState::Error(e.to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            style: "font-family: 'Segoe UI', 'Hiragino Sans', sans-serif; height: 480px; display: flex; flex-direction: column; background: {BG}; overflow: hidden; color: {TEXT};",

            // Header（ドラッグ領域）
            div {
                style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; background: {SURFACE}; border-bottom: 1px solid {BORDER}; cursor: grab; user-select: none;",
                onmousedown: move |_| { let _ = win_drag.drag_window(); },
                span {
                    style: "font-size: 14px; font-weight: 600; color: {TEXT_DIM}; letter-spacing: 0.04em;",
                    "WebSearch Launcher"
                }
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    button {
                        style: "background: none; border: none; cursor: pointer; font-size: 16px; color: {TEXT_DIM}; padding: 2px 6px; line-height: 1; border-radius: 4px;",
                        title: "設定",
                        onmousedown: |e| e.stop_propagation(),
                        onclick: move |_| { screen.set(Screen::Settings); },
                        "⚙"
                    }
                    button {
                        style: "background: none; border: none; cursor: pointer; font-size: 16px; color: {TEXT_DIM}; padding: 2px 6px; line-height: 1; border-radius: 4px;",
                        title: "最小化",
                        onmousedown: |e| e.stop_propagation(),
                        onclick: move |_| { win_minimize.set_minimized(true); },
                        "−"
                    }
                    button {
                        style: "background: none; border: none; cursor: pointer; font-size: 14px; color: {TEXT_DIM}; padding: 2px 6px; line-height: 1; border-radius: 4px;",
                        title: "閉じる",
                        onmousedown: |e| e.stop_propagation(),
                        onclick: move |_| { win_close.close(); },
                        "✕"
                    }
                }
            }

            // Search bar
            div {
                style: "padding: 12px 16px; background: {SURFACE}; border-bottom: 1px solid {BORDER};",
                div {
                    style: "display: flex; gap: 8px;",
                    input {
                        style: "flex: 1; padding: 8px 12px; background: {SURFACE2}; border: 1px solid {BORDER}; border-radius: 6px; font-size: 14px; outline: none; color: {TEXT}; caret-color: {ACCENT};",
                        placeholder: "検索キーワードを入力して Enter...",
                        value: "{query}",
                        oninput: move |e| { query.set(e.value()); },
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                do_search();
                            }
                        },
                    }
                    button {
                        style: "padding: 8px 18px; background: {ACCENT}; color: #fff; border: none; border-radius: 6px; font-size: 14px; cursor: pointer; white-space: nowrap; font-weight: 500;",
                        onclick: move |_| { do_search(); },
                        "検索"
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 6px; margin-top: 8px;",
                    input {
                        r#type: "checkbox",
                        id: "news_toggle",
                        checked: "{use_news}",
                        onchange: move |e| { use_news.set(e.checked()); },
                    }
                    label {
                        r#for: "news_toggle",
                        style: "font-size: 13px; color: {TEXT_DIM}; cursor: pointer; user-select: none;",
                        "ニュースソース優先で検索する"
                    }
                }
            }

            // Content area
            div {
                style: "flex: 1; overflow-y: auto; padding: 12px 16px;",
                match app_state() {
                    AppState::Idle => rsx! {
                        div {
                            style: "color: {TEXT_DIM}; font-size: 14px; text-align: center; margin-top: 60px;",
                            "キーワードを入力して検索してください"
                        }
                    },
                    AppState::Loading => rsx! {
                        div {
                            style: "color: {ACCENT}; font-size: 14px; text-align: center; margin-top: 60px;",
                            "検索・要約中..."
                        }
                    },
                    AppState::Error(msg) => rsx! {
                        div {
                            style: "background: {ERROR_BG}; border: 1px solid {ERROR_BORDER}; border-radius: 6px; padding: 12px; color: {ERROR_TEXT}; font-size: 13px; white-space: pre-wrap; line-height: 1.6;",
                            "{msg}"
                        }
                    },
                    AppState::Done { summary, sources } => rsx! {
                        div {
                            div {
                                style: "background: {SURFACE}; border: 1px solid {BORDER}; border-radius: 6px; padding: 12px; margin-bottom: 10px;",
                                div {
                                    style: "font-size: 11px; font-weight: 700; color: {ACCENT}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.08em;",
                                    "要約"
                                }
                                div {
                                    style: "font-size: 13px; color: {TEXT}; line-height: 1.75; white-space: pre-wrap;",
                                    "{summary}"
                                }
                            }
                            div {
                                style: "background: {SURFACE}; border: 1px solid {BORDER}; border-radius: 6px; padding: 12px;",
                                div {
                                    style: "font-size: 11px; font-weight: 700; color: {TEXT_DIM}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.08em;",
                                    "参照元"
                                }
                                for (title, url) in &sources {
                                    SourceLink { key: "{url}", title: title.clone(), url: url.clone() }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn SourceLink(title: String, url: String) -> Element {
    let url_clone = url.clone();
    rsx! {
        div {
            style: "margin-bottom: 6px;",
            a {
                style: "font-size: 12px; color: {ACCENT}; text-decoration: none; cursor: pointer; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; opacity: 0.85;",
                title: "{url}",
                onclick: move |_| { open::that(&url_clone).ok(); },
                "↗ {title}"
            }
        }
    }
}

#[component]
fn SettingsScreen(screen: Signal<Screen>, cfg: Signal<Settings>) -> Element {
    let mut host_input = use_signal(|| cfg.read().ollama_host.clone());
    let mut model_input = use_signal(|| cfg.read().model_name.clone());
    let mut key_input = use_signal(|| cfg.read().tavily_api_key.clone());
    let mut save_msg = use_signal(|| Option::<String>::None);
    let win = use_window();
    let win_drag = win.clone();
    let win_minimize = win.clone();
    let win_close = win.clone();

    let label_style = format!("display: block; font-size: 13px; font-weight: 600; color: {TEXT}; margin-bottom: 6px;");
    let input_style = format!("width: 100%; padding: 8px 12px; background: {SURFACE2}; border: 1px solid {BORDER}; border-radius: 6px; font-size: 14px; box-sizing: border-box; color: {TEXT}; outline: none; caret-color: {ACCENT};");
    let hint_style = format!("font-size: 11px; color: {TEXT_DIM}; margin-top: 4px;");

    rsx! {
        div {
            style: "font-family: 'Segoe UI', 'Hiragino Sans', sans-serif; height: 480px; display: flex; flex-direction: column; background: {BG}; color: {TEXT};",

            // Header（ドラッグ領域）
            div {
                style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; background: {SURFACE}; border-bottom: 1px solid {BORDER}; cursor: grab; user-select: none;",
                onmousedown: move |_| { let _ = win_drag.drag_window(); },
                button {
                    style: "background: none; border: none; cursor: pointer; font-size: 14px; color: {ACCENT}; padding: 2px 6px; border-radius: 4px;",
                    onmousedown: |e| e.stop_propagation(),
                    onclick: move |_| { screen.set(Screen::Search); },
                    "← 戻る"
                }
                span {
                    style: "font-size: 14px; font-weight: 600; color: {TEXT_DIM}; letter-spacing: 0.04em;",
                    "設定"
                }
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    button {
                        style: "background: none; border: none; cursor: pointer; font-size: 16px; color: {TEXT_DIM}; padding: 2px 6px; line-height: 1; border-radius: 4px;",
                        title: "最小化",
                        onmousedown: |e| e.stop_propagation(),
                        onclick: move |_| { win_minimize.set_minimized(true); },
                        "−"
                    }
                    button {
                        style: "background: none; border: none; cursor: pointer; font-size: 14px; color: {TEXT_DIM}; padding: 2px 6px; border-radius: 4px;",
                        title: "閉じる",
                        onmousedown: |e| e.stop_propagation(),
                        onclick: move |_| { win_close.close(); },
                        "✕"
                    }
                }
            }

            div {
                style: "flex: 1; overflow-y: auto; padding: 20px 16px;",

                div {
                    style: "margin-bottom: 20px;",
                    label {
                        style: "{label_style}",
                        "Tavily API キー"
                    }
                    input {
                        style: "{input_style} font-family: monospace;",
                        r#type: "password",
                        placeholder: "tvly-xxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                        value: "{key_input}",
                        oninput: move |e| { key_input.set(e.value()); },
                    }
                    div {
                        style: "{hint_style}",
                        "空欄の場合は環境変数 TAVILY_API_KEY を使用します"
                    }
                }

                div {
                    style: "margin-bottom: 20px;",
                    label {
                        style: "{label_style}",
                        "Ollama ホスト"
                    }
                    input {
                        style: "{input_style}",
                        placeholder: "http://localhost:11434",
                        value: "{host_input}",
                        oninput: move |e| { host_input.set(e.value()); },
                    }
                    div {
                        style: "{hint_style}",
                        "例: http://localhost:11434 または http://192.168.1.100:11434"
                    }
                }

                div {
                    style: "margin-bottom: 28px;",
                    label {
                        style: "{label_style}",
                        "モデル名"
                    }
                    input {
                        style: "{input_style}",
                        placeholder: "qwen2.5:7b",
                        value: "{model_input}",
                        oninput: move |e| { model_input.set(e.value()); },
                    }
                    div {
                        style: "{hint_style}",
                        "例: qwen2.5:7b, llama3.2:3b, gemma3:4b"
                    }
                }

                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    button {
                        style: "padding: 9px 24px; background: {ACCENT}; color: #fff; border: none; border-radius: 6px; font-size: 14px; cursor: pointer; font-weight: 500;",
                        onclick: move |_| {
                            let new_cfg = Settings {
                                tavily_api_key: key_input.read().trim().to_string(),
                                ollama_host: host_input.read().trim().to_string(),
                                model_name: model_input.read().trim().to_string(),
                            };
                            match settings::save(&new_cfg) {
                                Ok(_) => {
                                    cfg.set(new_cfg);
                                    save_msg.set(Some("保存しました".to_string()));
                                }
                                Err(e) => {
                                    save_msg.set(Some(format!("保存に失敗しました: {}", e)));
                                }
                            }
                        },
                        "保存"
                    }
                    if let Some(msg) = save_msg() {
                        div {
                            style: "font-size: 13px; color: {SUCCESS};",
                            "{msg}"
                        }
                    }
                }
            }
        }
    }
}
