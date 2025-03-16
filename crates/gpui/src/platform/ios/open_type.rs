#![allow(unused, non_upper_case_globals)]

use crate::{FontFallbacks, FontFeatures};
use font_kit::font::Font as FontKitFont;
use objc2::{rc::Retained, runtime::AnyObject};
use objc2_core_foundation::{CFLocaleCopyPreferredLanguages, CFRetained, CFString, Type};
use objc2_core_text::{
    kCTFontFeatureSettingsAttribute, kCTFontOpenTypeFeatureTag, kCTFontOpenTypeFeatureValue,
    CTFont, CTFontCopyDefaultCascadeListForLanguages, CTFontCreateCopyWithAttributes,
    CTFontDescriptor, CTFontDescriptorCreateWithAttributes, CTFontDescriptorCreateWithNameAndSize,
};
use objc2_foundation::{ns_string, NSArray, NSDictionary, NSString};
use std::{ffi::c_void, ptr};

// This should be a util function but it's currently only needed here.
// https://github.com/madsmtm/objc2/issues/693
// Due to orphan rule, we cannot use `From` trait.
fn to_nsstring(cf_string: CFRetained<CFString>) -> Retained<NSString> {
    Into::<Retained<NSString>>::into(cf_string)
        .downcast::<NSString>()
        .unwrap()
}

fn generate_feature_array(features: &FontFeatures) -> Vec<Retained<AnyObject>> {
    let mut feature_array: Vec<Retained<AnyObject>> = Vec::new();

    for (tag, value) in features.tag_value_list() {
        let dict = NSDictionary::from_retained_objects(
            &[
                &to_nsstring(unsafe { kCTFontOpenTypeFeatureTag.retain() }),
                &to_nsstring(unsafe { kCTFontOpenTypeFeatureValue.retain() }),
            ],
            &[&NSString::from_str(&tag), &NSNumber::from_u32(*value)],
        );

        feature_array.push(dict.downcast::<AnyObject>().unwrap());
    }

    feature_array
}

fn generate_fallback_array(
    fallbacks: &FontFallbacks,
    font: &CTFont,
) -> Vec<Retained<CTFontDescriptor>> {
    let mut fallback_array: Vec<Retained<CTFontDescriptor>> = Vec::new();

    for name in fallbacks.fallback_list() {
        fallback_array.push(unsafe {
            CTFontDescriptorCreateWithNameAndSize(&CFString::from_str(name), 0.0).into()
        })
    }

    let preferred_languages = unsafe { CFLocaleCopyPreferredLanguages() };
    let default_fallbacks =
        unsafe { CTFontCopyDefaultCascadeListForLanguages(font, *preferred_languages) };

    if let Some(default_fallbacks) = default_fallbacks {
        let default_fallbacks = default_fallbacks.downcast::<NSArray>().unwrap();
        for fallback in default_fallbacks.iter() {
            fallback_array.push(fallback.downcast::<CTFontDescriptor>().unwrap());
        }
    }

    fallback_array
}

pub fn apply_features_and_fallbacks(
    font: &mut FontKitFont,
    features: &FontFeatures,
    fallbacks: Option<&FontFallbacks>,
) -> anyhow::Result<()> {
    let mut keys = vec![&to_nsstring(unsafe {
        kCTFontFeatureSettingsAttribute.retain()
    })];
    let mut values = vec![generate_feature_array(features)];

    if let Some(fallbacks) = fallbacks {
        if !fallbacks.fallback_list().is_empty() {
            keys.push(&to_nsstring(unsafe {
                kCTFontCascadeListAttribute.retain()
            }));
            values.push(generate_fallback_array(fallbacks, font.native_font()));
        }
    }

    let attrs = NSDictionary::from_retained_objects(&keys, &values);
    let descriptor = unsafe { CTFontDescriptorCreateWithAttributes(attrs) };
    let font = unsafe {
        CTFontCreateCopyWithAttributes(font.native_font(), 0.0, std::ptr::null(), descriptor)
    };

    *font = unsafe { FontKitFont::from_native_font(&font) };

    Ok(())
}
