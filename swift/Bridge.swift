// The app side of nsh-widgets on Apple platforms, called from Rust over the C ABI.
import Foundation
import WidgetKit

@_cdecl("nsh_widgets_reload_kind")
public func nsh_widgets_reload_kind(_ kind: UnsafePointer<CChar>) {
    WidgetCenter.shared.reloadTimelines(ofKind: String(cString: kind))
    #if os(iOS)
    // Controls (iOS 18) read the same data: redraw them with the widgets.
    if #available(iOS 18.0, *) { ControlCenter.shared.reloadAllControls() }
    #endif
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

// Live Activities (iOS 16.2+). The attributes must match NshActivities.swift in the widget extension:
// ActivityKit pairs the app's and the extension's types by name.
#if os(iOS) && canImport(ActivityKit)
// Weak: ActivityKit is iOS 16.1+ and NSH apps run from iOS 15.
@_weakLinked import ActivityKit

struct NshActivityAttributes: ActivityAttributes {
    struct ContentState: Codable, Hashable {
        var data: String
    }
    var kind: String
}
#endif

/// 0: started, 1: Live Activities do not exist here, 2: the user turned them off, 3: ActivityKit refused.
@_cdecl("nsh_widgets_activity_start")
public func nsh_widgets_activity_start(_ kind: UnsafePointer<CChar>, _ data: UnsafePointer<CChar>) -> Int32 {
    #if os(iOS) && canImport(ActivityKit)
    if #available(iOS 16.2, *) {
        guard ActivityAuthorizationInfo().areActivitiesEnabled else { return 2 }
        let (k, d) = (String(cString: kind), String(cString: data))
        // One activity per kind: a new start replaces the running one.
        for a in Activity<NshActivityAttributes>.activities where a.attributes.kind == k {
            Task { await a.end(nil, dismissalPolicy: .immediate) }
        }
        do {
            _ = try Activity.request(attributes: NshActivityAttributes(kind: k),
                                     content: .init(state: .init(data: d), staleDate: nil), pushType: nil)
            return 0
        } catch {
            return 3
        }
    }
    #endif
    return 1
}

/// 0: updated, 1: unavailable, 4: no activity of that kind is running.
@_cdecl("nsh_widgets_activity_update")
public func nsh_widgets_activity_update(_ kind: UnsafePointer<CChar>, _ data: UnsafePointer<CChar>) -> Int32 {
    #if os(iOS) && canImport(ActivityKit)
    if #available(iOS 16.2, *) {
        let (k, d) = (String(cString: kind), String(cString: data))
        let running = Activity<NshActivityAttributes>.activities.filter { $0.attributes.kind == k }
        guard !running.isEmpty else { return 4 }
        for a in running { Task { await a.update(.init(state: .init(data: d), staleDate: nil)) } }
        return 0
    }
    #endif
    return 1
}

/// 0: ended (or none was running), 1: unavailable.
@_cdecl("nsh_widgets_activity_end")
public func nsh_widgets_activity_end(_ kind: UnsafePointer<CChar>) -> Int32 {
    #if os(iOS) && canImport(ActivityKit)
    if #available(iOS 16.2, *) {
        let k = String(cString: kind)
        for a in Activity<NshActivityAttributes>.activities where a.attributes.kind == k {
            Task { await a.end(nil, dismissalPolicy: .immediate) }
        }
        return 0
    }
    #endif
    return 1
}
