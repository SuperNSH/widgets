// Live Activities for nsh-widgets (iOS only), compiled into the extension when widgets.json lists
// "activities". One generic activity type draws any kind: the lock-screen banner and the expanded
// Dynamic Island draw the kind's `root` tree, the compact and minimal Dynamic Island its `accessory`
// tree (a row's first child leads, the rest trail). Data comes from the activity's state, over data.json.
#if os(iOS)
import ActivityKit
import SwiftUI
import WidgetKit

/// Must match the app's copy in the nsh-widgets crate's Bridge.swift: ActivityKit pairs them by name.
struct NshActivityAttributes: ActivityAttributes {
    struct ContentState: Codable, Hashable {
        /// The activity's data, a JSON object, filled into the tree's {key} templates.
        var data: String
    }
    var kind: String
}

private func activityData(_ kind: String, _ state: String) -> [String: Any] {
    var data = NshUIStore.data(kind)
    if let json = try? JSONSerialization.jsonObject(with: Data(state.utf8)) as? [String: Any] {
        data.merge(json) { $1 }
    }
    return data
}

private struct ActivityTree: View {
    let node: UINode?
    let data: [String: Any]
    let kind: String
    let font: String?
    var body: some View {
        if let node {
            NodeView(node: UINode(t: node.t, style: UIStyle(fg: node.style?.fg, size: node.style?.size, weight: node.style?.weight,
                                                            gap: node.style?.gap, align: node.style?.align, grow: node.style?.grow),
                                  text: node.text, action: node.action, children: node.children, key: node.key, mode: node.mode,
                                  name: node.name, interval: node.interval, max: node.max),
                     data: data, kind: kind, font: font, date: Date())
        }
    }
}

private func pieces(_ node: UINode?) -> (UINode?, UINode?) {
    guard let node else { return (nil, nil) }
    if node.t == "row", let kids = node.children, kids.count > 1 {
        return (kids[0], UINode(t: "row", style: node.style, children: Array(kids.dropFirst())))
    }
    return (node, nil)
}

private func activityBackground(_ style: UIStyle?, _ scheme: ColorScheme) -> AnyShapeStyle {
    let pick = { (p: [String]) in Color(hex: scheme == .dark ? p[1] : p[0]) }
    if let g = style?.gradient {
        let (a, b): (UnitPoint, UnitPoint) = g.dir == "r" ? (.leading, .trailing) : g.dir == "br" ? (.topLeading, .bottomTrailing) : (.top, .bottom)
        return AnyShapeStyle(LinearGradient(colors: [pick(g.from), pick(g.to)], startPoint: a, endPoint: b))
    }
    if let bg = style?.bg, bg.count == 2 { return AnyShapeStyle(pick(bg)) }
    return AnyShapeStyle(Color.clear)
}

private struct BannerView: View {
    let kind: String
    let state: String
    @Environment(\.colorScheme) private var scheme
    var body: some View {
        let snap = NshUIStore.snapshot(kind)
        let p = snap?.root.style?.pad ?? [16, 16, 16, 16]
        ActivityTree(node: snap?.root, data: activityData(kind, state), kind: kind, font: snap?.font)
            .padding(EdgeInsets(top: p[0], leading: p[3], bottom: p[2], trailing: p[1]))
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(activityBackground(snap?.root.style, scheme))
    }
}

struct NshActivityWidget: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: NshActivityAttributes.self) { context in
            BannerView(kind: context.attributes.kind, state: context.state.data)
                .activityBackgroundTint(nil)
        } dynamicIsland: { context in
            let kind = context.attributes.kind
            let snap = NshUIStore.snapshot(kind)
            let data = activityData(kind, context.state.data)
            let (lead, trail) = pieces(snap?.accessory ?? snap?.root)
            return DynamicIsland {
                DynamicIslandExpandedRegion(.bottom) {
                    // The island's corners curve in: keep the tree clear of them.
                    ActivityTree(node: snap?.root, data: data, kind: kind, font: snap?.font)
                        .padding(.horizontal, 8)
                }
            } compactLeading: {
                ActivityTree(node: lead, data: data, kind: kind, font: snap?.font)
            } compactTrailing: {
                ActivityTree(node: trail, data: data, kind: kind, font: snap?.font)
            } minimal: {
                ActivityTree(node: lead, data: data, kind: kind, font: snap?.font)
            }
        }
    }
}
#endif
