use std::time::Duration;

use cosmic::iced::Rotation;
use cosmic::widget::Icon;
use cosmic::widget::Id as CosmicId;
use cosmic::widget::{icon, icon::Handle};
use cosmic_time::timeline::{self, Interped};
use cosmic_time::{timeline::Frame, Timeline};
use cosmic_time::{Cubic, Ease, MovementType, Repeat};

static REFRESH_ICON_HANDLE: std::sync::LazyLock<Handle> =
    std::sync::LazyLock::new(|| icon::from_name("view-refresh-symbolic").into());

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(CosmicId);

impl Id {
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(CosmicId::new(id))
    }

    pub fn unique() -> Self {
        Self(CosmicId::unique())
    }

    pub fn into_chain(self) -> Chain {
        Chain::new(self)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_widget(self, timeline: &Timeline, size: u16) -> Icon {
        RefreshIcon::as_widget(&self, timeline, size)
    }
}

impl From<Id> for CosmicId {
    fn from(value: Id) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct Chain {
    id: Id,
    links: Vec<RefreshIcon>,
}

impl Chain {
    pub fn new(id: Id) -> Self {
        let links = (1..=72) // doubled the steps from 36 to 72 for smoother animation
            .map(|i| {
                let angle = i as f32 * 5.0; // reduced step size from 10 to 5 degrees
                RefreshIcon::new(Duration::from_millis(50)) // shorter duration per frame
                    .angle(angle)
            })
            .collect();

        Chain { id, links }
    }
}

impl From<Chain> for timeline::Chain {
    fn from(chain: Chain) -> Self {
        timeline::Chain::new(
            chain.id.0,
            Repeat::Forever,
            chain
                .links
                .into_iter()
                .map(std::convert::Into::into)
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RefreshIcon {
    at: MovementType,
    ease: Ease,
    angle: f32,
}

impl RefreshIcon {
    pub fn new(at: impl Into<MovementType>) -> RefreshIcon {
        let at = at.into();
        RefreshIcon {
            at,
            ease: Cubic::InOut.into(),
            angle: 0.0,
        }
    }

    pub fn angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    pub fn as_widget(id: &Id, timeline: &Timeline, size: u16) -> Icon {
        let angle = if let Some(Interped { value, .. }) = timeline.get(&id.0, 0) {
            value
        } else {
            0.0
        };
        icon(REFRESH_ICON_HANDLE.clone())
            .rotation(Rotation::Floating((angle.to_radians()).into()))
            .size(size)
    }
}

impl From<RefreshIcon> for Vec<Option<Frame>> {
    fn from(icon: RefreshIcon) -> Vec<Option<Frame>> {
        vec![Some(Frame::lazy(icon.at, icon.angle, icon.ease))]
    }
}
