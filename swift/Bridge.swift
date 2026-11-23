// The app side of nsh-widgets on Apple platforms, called from Rust over the C ABI.
import Foundation
import WidgetKit

@_cdecl("nsh_widgets_reload_kind")
public func nsh_widgets_reload_kind(_ kind: UnsafePointer<CChar>) {
    WidgetCenter.shared.reloadTimelines(ofKind: String(cString: kind))
}

@_cdecl("nsh_widgets_reload_all")
public func nsh_widgets_reload_all() {
    WidgetCenter.shared.reloadAllTimelines()
}

/// The App Group container's path, or nil when the app lacks the group's entitlement.
@_cdecl("nsh_widgets_container_path")
public func nsh_widgets_container_path(_ group: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>? {
    guard let url = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: String(cString: group)) else { return nil }
    return strdup(url.path)
}

/// The App Group `nsh-widgets build` wrote into the app's Info.plist, or nil.
@_cdecl("nsh_widgets_info_group")
public func nsh_widgets_info_group() -> UnsafeMutablePointer<CChar>? {
    guard let group = Bundle.main.object(forInfoDictionaryKey: "NshWidgetsAppGroup") as? String, !group.isEmpty else { return nil }
    return strdup(group)
}

// The widget's button intent posts a Darwin notification; a running app counts them to wake at once.
private let changeQueue = DispatchQueue(label: "nsh.widgets.changes")
private var changeCount = 0
private var observing = Set<String>()

@_cdecl("nsh_widgets_observe")
public func nsh_widgets_observe(_ name: UnsafePointer<CChar>) {
    let key = String(cString: name)
    guard changeQueue.sync(execute: { observing.insert(key).inserted }) else { return }
    CFNotificationCenterAddObserver(CFNotificationCenterGetDarwinNotifyCenter(), nil, { _, _, _, _, _ in
        changeQueue.sync { changeCount += 1 }
    }, key as CFString, nil, .deliverImmediately)
}

@_cdecl("nsh_widgets_change_count")
public func nsh_widgets_change_count() -> Int {
    changeQueue.sync { changeCount }
}
