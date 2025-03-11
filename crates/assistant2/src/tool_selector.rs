use std::sync::Arc;

use assistant_tool::{ToolSource, ToolWorkingSet};
use gpui::Entity;
use scripting_tool::ScriptingTool;
use ui::{prelude::*, ContextMenu, IconButtonShape, PopoverMenu, Tooltip};

pub struct ToolSelector {
    tools: Arc<ToolWorkingSet>,
}

impl ToolSelector {
    pub fn new(tools: Arc<ToolWorkingSet>, _cx: &mut Context<Self>) -> Self {
        Self { tools }
    }

    fn build_context_menu(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<ContextMenu> {
        ContextMenu::build(window, cx, |mut menu, _window, cx| {
            let tools_by_source = self.tools.tools_by_source(cx);

            for (source, tools) in tools_by_source {
                let mut tools = tools
                    .into_iter()
                    .map(|tool| {
                        let source = tool.source();
                        let name = tool.name().into();
                        let is_enabled = self.tools.is_enabled(&source, &name);

                        (source, name, is_enabled)
                    })
                    .collect::<Vec<_>>();

                if ToolSource::Native == source {
                    tools.push((
                        ToolSource::Native,
                        ScriptingTool::NAME.into(),
                        self.tools.is_scripting_tool_enabled(),
                    ));
                    tools.sort_by(|(_, name_a, _), (_, name_b, _)| name_a.cmp(name_b));
                }

                menu = match source {
                    ToolSource::Native => menu.header("Zed"),
                    ToolSource::ContextServer { id } => menu.separator().header(id),
                };

                for (source, name, is_enabled) in tools {
                    menu =
                        menu.toggleable_entry(name.clone(), is_enabled, IconPosition::End, None, {
                            let tools = self.tools.clone();
                            move |_window, _cx| {
                                if name.as_ref() == ScriptingTool::NAME {
                                    if is_enabled {
                                        tools.disable_scripting_tool();
                                    } else {
                                        tools.enable_scripting_tool();
                                    }
                                } else {
                                    if is_enabled {
                                        tools.disable(source.clone(), &[name.clone()]);
                                    } else {
                                        tools.enable(source.clone(), &[name.clone()]);
                                    }
                                }
                            }
                        });
                }
            }

            menu
        })
    }
}

impl Render for ToolSelector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let this = cx.entity().clone();
        PopoverMenu::new("tool-selector")
            .menu(move |window, cx| {
                Some(this.update(cx, |this, cx| this.build_context_menu(window, cx)))
            })
            .trigger_with_tooltip(
                IconButton::new("tool-selector-button", IconName::SettingsAlt)
                    .shape(IconButtonShape::Square)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Muted),
                Tooltip::text("Customize Tools"),
            )
            .anchor(gpui::Corner::BottomLeft)
    }
}
