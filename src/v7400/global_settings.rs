//! Document-wide settings.

use crate::v7400::axis::{AxisSystem, SignedAxis};
use crate::v7400::properties::PropertiesNodeId;
use crate::v7400::property::loaders::PrimitiveLoader;
use crate::v7400::{Document, ObjectProperties, Result};

/// A proxy to document-wide settings.
#[derive(Debug, Clone, Copy)]
pub struct GlobalSettings<'a> {
    /// Objects properties of `/GlobalSettings` node.
    props: ObjectProperties<'a>,
}

impl<'a> GlobalSettings<'a> {
    /// Creates a new proxy to `GlobalSettings` props.
    pub(super) fn new(doc: &'a Document) -> Result<Self> {
        let global_settings_node = doc
            .tree()
            .root()
            .first_child_by_name("GlobalSettings")
            .ok_or_else(|| error!("expected `/GlobalSettings` node but not found"))?;
        let direct_props = global_settings_node
            .first_child_by_name("Properties70")
            .map(|node| PropertiesNodeId::new(node.node_id()));

        // I am not confident about native typename being `FbxGlobalSettings`,
        // but it seems likely.
        // Documents I investigated has no default properties (`PropertyTemplate`) for any FB
        let default_props = doc
            .definitions_cache()
            .props_node_id("GlobalSettings", "FbxGlobalSettings");
        Ok(Self {
            props: ObjectProperties::new(direct_props, default_props, doc),
        })
    }

    /// Returns the axis system.
    pub fn axis_system(&self) -> Result<AxisSystem> {
        let up = self.up_axis()?;
        let front = self.front_axis()?;
        let right = self.right_axis()?;

        AxisSystem::from_up_front_right(up, front, right).ok_or_else(|| {
            error!(
                "invalid axis system: (up, front, right) = ({}, {}, {})",
                up, front, right
            )
        })
    }

    /// Returns the raw properties.
    ///
    /// This would be useful when the user wants to access to properties
    /// not supported by this crate.
    #[inline]
    #[must_use]
    pub fn raw_properties(&self) -> &ObjectProperties<'a> {
        &self.props
    }

    /// Returns the up axis.
    pub fn up_axis(&self) -> Result<SignedAxis> {
        load_axis_from_prop("Up", self.up_axis_raw()?, self.up_axis_sign_raw()?)
    }

    /// Returns the front axis.
    pub fn front_axis(&self) -> Result<SignedAxis> {
        load_axis_from_prop("Front", self.front_axis_raw()?, self.front_axis_sign_raw()?)
    }

    /// Returns the "coord axis" (i.e. rightward axis).
    pub fn right_axis(&self) -> Result<SignedAxis> {
        load_axis_from_prop("Coord", self.coord_axis_raw()?, self.coord_axis_sign_raw()?)
    }

    /// Returns the raw `UpAxis` value.
    fn up_axis_raw(&self) -> Result<i32> {
        self.props
            .get("UpAxis")
            .ok_or_else(|| error!("expected `UpAxis` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Returns the raw `UpAxisSign` value.
    fn up_axis_sign_raw(&self) -> Result<i32> {
        self.props
            .get("UpAxisSign")
            .ok_or_else(|| error!("expected `UpAxisSign` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Return the raws `FrontAxis` value.
    fn front_axis_raw(&self) -> Result<i32> {
        self.props
            .get("FrontAxis")
            .ok_or_else(|| error!("expected `FrontAxis` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Returns the raw `FrontAxisSign` value.
    fn front_axis_sign_raw(&self) -> Result<i32> {
        self.props
            .get("FrontAxisSign")
            .ok_or_else(|| error!("expected `FrontAxisSign` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Returns the raw `CoordAxis` value.
    fn coord_axis_raw(&self) -> Result<i32> {
        self.props
            .get("CoordAxis")
            .ok_or_else(|| error!("expected `CoordAxis` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Returns the raw `CoordAxisSign` value.
    fn coord_axis_sign_raw(&self) -> Result<i32> {
        self.props
            .get("CoordAxisSign")
            .ok_or_else(|| error!("expected `CoordAxisSign` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Returns the "original up axis".
    pub fn original_up_axis(&self) -> Result<SignedAxis> {
        load_axis_from_prop(
            "OriginalUp",
            self.original_up_axis_raw()?,
            self.original_up_axis_sign_raw()?,
        )
    }

    /// Returns the raw `OriginalUpAxis` value.
    fn original_up_axis_raw(&self) -> Result<i32> {
        self.props
            .get("OriginalUpAxis")
            .ok_or_else(|| error!("expected `OriginalUpAxis` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Returns the raw `OriginalUpAxisSign` value.
    fn original_up_axis_sign_raw(&self) -> Result<i32> {
        self.props
            .get("OriginalUpAxisSign")
            .ok_or_else(|| error!("expected `OriginalUpAxisSign` property but not found"))?
            .value(PrimitiveLoader::<i32>::new())
    }

    /// Returns the unit scale factor.
    ///
    /// Unit scale factor is a scale factor from unit length in the document to
    /// the real world length.
    /// For example, a unit scale factor contains the information such as
    /// "length 1.0 in the document corresponds to 1 cm in the real world".
    ///
    /// Note that some third-party software may ignore this property when they
    /// load the document. This may lead to unexpected difference between the
    /// results by such implementations and by softwares which apply the factor.
    /// Developers should decide carefully whether to apply the unit scale
    /// factor or not.
    pub fn unit_scale_factor(&self) -> Result<UnitScaleFactor> {
        self.unit_scale_factor_raw().and_then(UnitScaleFactor::new)
    }

    /// Returns the raw unit scale factor.
    pub fn unit_scale_factor_raw(&self) -> Result<f64> {
        self.props
            .get("UnitScaleFactor")
            .ok_or_else(|| error!("expected `UnitScaleFactor` property but not found"))?
            .value(PrimitiveLoader::<f64>::new())
    }
}

/// Loads a signed axis from the given property values for axis and axis sign.
#[inline]
fn load_axis_from_prop(axis_name: &str, axis: i32, axis_sign: i32) -> Result<SignedAxis> {
    match (axis, axis_sign) {
        (0, 1) => Ok(SignedAxis::PosX),
        (0, -1) => Ok(SignedAxis::NegX),
        (1, 1) => Ok(SignedAxis::PosY),
        (1, -1) => Ok(SignedAxis::NegY),
        (2, 1) => Ok(SignedAxis::PosZ),
        (2, -1) => Ok(SignedAxis::NegZ),
        _ => {
            if !(0..=2).contains(&axis) {
                return Err(error!(
                    "invalid `{}Axis` property value: expected 0, 1, or 2 but got {}",
                    axis_name, axis
                ));
            }
            if (axis_sign == 1) || (axis_sign == -1) {
                return Err(error!(
                    "invalid `{}AxisSign` property value: expected 1 or -1, but got {}",
                    axis_name, axis_sign
                ));
            }

            unreachable!(
                "at least one of axis or axis sign must be invalid: \
                axis_name={:?}, axis={:?}, axis_sign={:?}",
                axis_name, axis, axis_sign
            );
        }
    }
}

/// Unit scale factor.
///
/// About unit scale factor, see the documentation for
/// [`GlobalSettings::unit_scale_factor`] method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitScaleFactor {
    /// A unit in the document in centimeters.
    unit_in_centimeters: f64,
}

impl UnitScaleFactor {
    /// Creates a new `UnitScaleFactor`.
    ///
    /// # Failures
    ///
    /// Fails if the given value is ["normal" floating-point number value][normal].
    /// In other words, fails if the given value is any of zero, infinite,
    /// subnormal, and NaN.
    ///
    /// [normal]: `std::num::FpCategory::Normal`
    pub fn new(unit_in_centimeters: f64) -> Result<Self> {
        // The scale should be neither zero, infinite, subnormal, or NaN.
        if !unit_in_centimeters.is_normal() {
            return Err(error!(
                "Expected \"normal\" floating-point number, but got {:?}",
                unit_in_centimeters.classify()
            ));
        }

        Ok(Self {
            unit_in_centimeters,
        })
    }

    /// Returns the unit size used in the document in centimeters.
    ///
    /// If the unit size (i.e. length of 1.0) is 1 meter in the document, this
    /// returns 100.0 since the unit is 100.0 cm.
    ///
    /// Note that some third-party software may ignore this value when they load
    /// the document. See the documentation of
    /// [`GlobalSettings::unit_scale_factor`] method for detail.
    #[inline(always)]
    #[must_use]
    pub fn unit_in_centimeters(self) -> f64 {
        self.unit_in_centimeters
    }
}