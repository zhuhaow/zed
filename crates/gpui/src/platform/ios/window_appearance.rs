use crate::WindowAppearance;
use objc2_ui_kit::UIUserInterfaceStyle;

impl WindowAppearance {
    pub(crate) unsafe fn from_native(style: UIUserInterfaceStyle) -> Self {
        if style == UIUserInterfaceStyle::Dark {
            Self::Dark
        } else {
            Self::Light
        }
    }
}
