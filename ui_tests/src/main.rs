#![no_main] // Required, we build this with `-bundle`.

use objc2::{ClassType, MainThreadOnly, define_class};
use objc2_foundation::NSString;
use objc2_foundation::{
    NSSearchPathDirectory, NSSearchPathDomainMask, NSSearchPathForDirectoriesInDomains,
};
use objc2_xc_test::XCTestCase;
use objc2_xc_ui_automation::{
    XCUIApplication, XCUIDevice, XCUIElementTypeQueryProvider, XCUIElementAttributes,
    XCUIScreenshot, XCUIScreenshotProviding,
};

define_class!(
    #[unsafe(super = XCTestCase)]
    #[thread_kind = MainThreadOnly]
    struct TestCase;

    impl TestCase {
        #[unsafe(method(setUp))]
        fn set_up(&self) {
            // Test setup code in here.
        }

        #[unsafe(method(tearDown))]
        fn tear_down(&self) {
            // Test teardown code in here.
        }

        #[unsafe(method(testScreenshot))]
        fn test_screenshot(&self) {
            let app = XCUIApplication::new(self.mtm());
            let device = XCUIDevice::sharedDevice(self.mtm());
            take_screenshots(&app, &device);
        }
    }
);

/// Load and initialize the class such that XCTest can see it.
#[ctor::ctor]
unsafe fn setup() {
    let _ = TestCase::class();
}

fn take_screenshots(app: &XCUIApplication, _device: &XCUIDevice) {
    use objc2_foundation::{NSDictionary, ns_string};

    // SIMCTL_CHILD_DINGHY_LLVM_PROFILE_FILE
    if let Ok(val) = std::env::var("DINGHY_LLVM_PROFILE_FILE") {
        let foo: objc2::rc::Retained<NSDictionary<NSString, NSString>> =
            NSDictionary::from_slices(
                &[
                    ns_string!("LLVM_PROFILE_FILE"),
                    ns_string!("LLVM_PROFILE_VERBOSE_ERRORS"),
                ],
                &[&NSString::from_str(val.as_str()), ns_string!("1")],
            );
        app.setLaunchEnvironment(&foo);
    }

    app.launch();

    // Screenshot 1: Initial state - empty TODO list
    save_screenshot(&app.screenshot(), "screenshot_initial".into());
    println!("=== Initial screenshot saved ===");

    // Find all text fields
    let text_fields = app.textFields();

    if text_fields.count() > 0 {
        // Tap the first text field (input)
        let input_field = text_fields.elementBoundByIndex(0);

        // Tap the input field
        input_field.tap();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Type a todo item
        input_field.typeText(&NSString::from_str("Buy groceries"));
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Screenshot after typing
        save_screenshot(&app.screenshot(), "screenshot_after_typing".into());
        println!("=== After typing screenshot saved ===");

        // Find and tap the Add button
        let all_buttons = app.buttons();

        // Try to find Add button by iterating
        let mut found_add = false;
        for i in 0..all_buttons.count() {
            let btn = all_buttons.elementBoundByIndex(i);
            let label = btn.label();
            let label_str = label.to_string();
            if label_str.contains("Add") {
                btn.tap();
                found_add = true;
                println!("Found and tapped Add button");
                break;
            }
        }

        if !found_add {
            // Try tapping the first button
            if all_buttons.count() > 0 {
                all_buttons.elementBoundByIndex(0).tap();
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Screenshot after adding
        save_screenshot(&app.screenshot(), "screenshot_after_add".into());
        println!("=== After adding todo screenshot saved ===");

        // Screenshot showing the todo item
        save_screenshot(&app.screenshot(), "screenshot_todo_added".into());
        println!("=== Todo added screenshot saved ===");
    }

    // Final screenshot
    std::thread::sleep(std::time::Duration::from_millis(2000));
    save_screenshot(&app.screenshot(), "screenshot_final".into());
    println!("=== Final screenshot saved ===");
}

fn save_screenshot(screenshot: &XCUIScreenshot, basename: String) {
    let path = NSSearchPathForDirectoriesInDomains(
        NSSearchPathDirectory::DocumentDirectory,
        NSSearchPathDomainMask::UserDomainMask,
        true,
    );
    if let Some(path) = path.firstObject() {
        let path = path.to_string();
        let path = std::path::Path::new(&path).join(format!("{basename}.png"));
        println!("PATH IS {path:?}");
        let res = screenshot
            .PNGRepresentation()
            .writeToFile_atomically(&NSString::from_str(path.to_str().unwrap()), false);
        assert!(res, "failed writing screenshot");
    }
}
