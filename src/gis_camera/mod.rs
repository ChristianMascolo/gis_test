mod systems;

use bevy::ui::Val;

pub struct Length(pub f32);

pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    fn from_width_height(width: Length, height: Length) -> Self {
        Size {
            width: (width.0),
            height: (height.0),
        }
    }

    pub fn to_bevy_size(&self) -> bevy::ui::Size {
        bevy::ui::Size::new(
            bevy::ui::Val::Px(self.width),
            bevy::ui::Val::Px(self.height),
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl Coord {
    fn to_dvec2(self) -> bevy::math::DVec2 {
        bevy::math::DVec2::new(self.x, self.y)
    }

    pub fn to_geo_coord(
        self,
        transform: &bevy::transform::components::Transform,
        window: &bevy::prelude::Window,
    ) -> geo::Coord {
        let size = bevy::math::DVec2::new(f64::from(window.width()), f64::from(window.height()));
        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = self.to_dvec2() - size / 2.0;
        // apply the camera transform
        let pos_wld = transform.compute_matrix().as_dmat4() * p.extend(0.0).extend(1.0);

        geo::Coord {
            x: pos_wld.x,
            y: pos_wld.y,
        }
    }
}

pub struct MyArea<'a> {
    pub window: &'a bevy::window::Window,
    pub left_offset_px: f32,
    pub top_offset_px: f32,
    pub right_offset_px: f32,
    pub bottom_offset_px: f32,
}

impl<'a> MyArea<'a> {
    fn top_left_screen_coord(&self) -> Coord {
        Coord {
            x: f64::from(self.left_offset_px),
            y: f64::from(self.top_offset_px),
        }
    }

    fn top_left_projected_geo_coord(
        &self,
        transform: &bevy::transform::components::Transform,
        window: &bevy::prelude::Window,
    ) -> geo::Coord {
        self.top_left_screen_coord().to_geo_coord(transform, window)
    }

    fn bottom_right_screen_coord(&self) -> Coord {
        Coord {
            x: f64::from(self.window.width() - self.right_offset_px),
            y: f64::from(self.window.height() - self.bottom_offset_px),
        }
    }

    fn bottom_right_projected_geo_coord(
        &self,
        transform: &bevy::transform::components::Transform,
        window: &bevy::prelude::Window,
    ) -> geo::Coord {
        self.bottom_right_screen_coord()
            .to_geo_coord(transform, window)
    }

    pub fn projected_geo_rect(
        &self,
        transform: &bevy::transform::components::Transform,
        window: &bevy::prelude::Window,
    ) -> geo::Rect {
        geo::Rect::new(
            self.top_left_projected_geo_coord(transform, window),
            self.bottom_right_projected_geo_coord(transform, window),
        )
    }

    fn width(&self) -> Length {
        Length(self.window.width() - self.left_offset_px - self.right_offset_px)
    }

    fn height(&self) -> Length {
        Length(self.window.height() - self.top_offset_px - self.bottom_offset_px)
    }

    pub fn size(&self) -> Size {
        Size::from_width_height(self.width(), self.height())
    }
}

#[derive(Clone, Copy)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    fn from_coord(coord: geo::Coord) -> Self {
        Offset {
            x: coord.x as f32,
            y: coord.y as f32,
        }
    }

    fn from_transform(transform: &bevy::prelude::Transform) -> Self {
        Offset {
            x: transform.translation.as_ref()[0],
            y: transform.translation.as_ref()[1],
        }
    }

    fn pan_x(&mut self, amount: f32, scale: Scale) {
        // what is the camera scale?
        self.x += amount * scale.0;
    }

    fn pan_y(&mut self, amount: f32, scale: Scale) {
        self.y += amount * scale.0;
    }

    fn to_transform_translation_vec(self) -> bevy::prelude::Vec3 {
        bevy::prelude::Vec3::new(
            self.x, self.y,
            999.9, // https://bevy-cheatbook.github.io/pitfalls/2d-camera-z.html
        )
    }
}

#[derive(Clone, Copy)]
pub struct Scale(pub f32);

impl Scale {
    fn from_transform(transform: &bevy::prelude::Transform) -> Self {
        Scale(transform.scale.as_ref()[0])
    }

    fn zoom(&mut self, amount: f32) {
        self.0 /= amount;
    }

    fn to_transform_scale_vec(self) -> bevy::prelude::Vec3 {
        bevy::prelude::Vec3::new(self.0, self.0, 1.)
    }
}

fn get_bevy_size(width: f32, height: f32) -> bevy::ui::Size {
    bevy::ui::Size::new(bevy::ui::Val::Px(width), bevy::ui::Val::Px(height))
}

fn determine_scale(win_width: f32, win_height: f32, bevy_size: bevy::ui::Size) -> f32 {
    let width: f32 = match bevy_size.width {
        Val::Px(p) => p,
        _ => unreachable!(),
    };
    let height: f32 = match bevy_size.height {
        Val::Px(p) => p,
        _ => unreachable!(),
    };
    (win_width / width).max(win_height / height)
}

fn set_camera_transform(transform: &mut bevy::prelude::Transform, offset: Offset, scale: Scale) {
    transform.translation = offset.to_transform_translation_vec();
    transform.scale = scale.to_transform_scale_vec();
}

pub struct MyCameraPlugin;

impl bevy::app::Plugin for MyCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system_set(systems::startup_system_set())
            .add_system_set(systems::system_set());
    }
}
