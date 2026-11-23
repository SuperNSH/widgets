// nsh-widgets renderer for iOS and macOS, compiled into the app's widget extension by `nsh-widgets build`.
// It knows nothing about any one widget: it draws whatever Widget Tree IR v1 tree the app wrote to
// snapshot.json, fills {key} templates from data.json, and runs the button action vocabulary itself so
// buttons work without the app. The App Group comes from the extension's Info.plist (NshWidgetsAppGroup).
import AppIntents
import SwiftUI
import WidgetKit

struct UIStyle: Decodable {
    var fg: [String]?
    var bg: [String]?
    var size: Double?
    var weight: String?
    var pad: [Double]?
    var gap: Double?
    var radius: Double?
    var align: String?
    var grow: Bool?
    var hide: [String]?
    var gradient: UIGradient?
    var animate: String?
    var height: Double?
}

struct UIGradient: Decodable {
    var from: [String]
    var to: [String]
    var dir: String
}

struct UINode: Decodable {
    var t: String
    var style: UIStyle?
    var text: String?
    var action: String?
    var children: [UINode]?
    var key: String?
    var mode: String?
    var name: String?
    var interval: Double?
}

struct UISnapshot: Decodable {
    var v: Int
    var font: String?
    var url: String?
    var root: UINode
}

let appGroup = Bundle.main.object(forInfoDictionaryKey: "NshWidgetsAppGroup") as? String ?? ""

enum NshUIStore {
    static func dir(_ kind: String) -> URL? {
        FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup)?
            .appendingPathComponent("nsh/widgets/\(kind)")
    }
    static func snapshot(_ kind: String) -> UISnapshot? {
        guard let f = dir(kind)?.appendingPathComponent("snapshot.json"), let d = try? Data(contentsOf: f) else { return nil }
        return try? JSONDecoder().decode(UISnapshot.self, from: d)
    }
    static func data(_ kind: String) -> [String: Any] {
        guard let f = dir(kind)?.appendingPathComponent("data.json"), let d = try? Data(contentsOf: f),
              let j = try? JSONSerialization.jsonObject(with: d) as? [String: Any] else { return [:] }
        return j["data"] as? [String: Any] ?? [:]
    }
    /// Runs one action from the vocabulary on data.json, queues it for the app and wakes a running app.
    static func apply(kind: String, action: String) {
        guard let d = dir(kind) else { return }
        var body = data(kind)
        let parts = action.split(separator: ":", maxSplits: 1).map(String.init)
        guard parts.count == 2, !appGroup.isEmpty else { return }
        let key = parts[1]
        switch parts[0] {
        case "inc": body[key] = ((body[key] as? Int) ?? 0) + 1
        case "dec": body[key] = ((body[key] as? Int) ?? 0) - 1
        case "toggle": body[key] = !((body[key] as? Bool) ?? false)
        default: return
        }
        try? FileManager.default.createDirectory(at: d, withIntermediateDirectories: true)
        let f = d.appendingPathComponent("data.json")
        if let out = try? JSONSerialization.data(withJSONObject: ["schema": 1, "data": body]) {
            let tmp = d.appendingPathComponent("data.json.tmp")
            try? out.write(to: tmp)
            if (try? FileManager.default.replaceItemAt(f, withItemAt: tmp)) == nil { try? FileManager.default.moveItem(at: tmp, to: f) }
        }
        let inbox = d.deletingLastPathComponent().appendingPathComponent("actions.jsonl")
        let event: [String: Any] = ["id": UUID().uuidString, "kind": kind, "action": action, "at": Int(Date().timeIntervalSince1970 * 1000)]
        guard let json = try? JSONSerialization.data(withJSONObject: event), var line = String(data: json, encoding: .utf8) else { return }
        line += "\n"
        if let h = try? FileHandle(forWritingTo: inbox) { h.seekToEndOfFile(); h.write(line.data(using: .utf8)!); try? h.close() }
        else { try? line.data(using: .utf8)!.write(to: inbox) }
        CFNotificationCenterPostNotification(CFNotificationCenterGetDarwinNotifyCenter(),
            CFNotificationName((appGroup + ".nsh.widgets.action") as CFString), nil, nil, true)
    }
}

struct NshActionIntent: AppIntent {
    static var title: LocalizedStringResource = "Widget action"
    static var isDiscoverable: Bool = false
    @Parameter(title: "Kind") var kind: String
    @Parameter(title: "Action") var action: String
    init() {}
    init(kind: String, action: String) {
        self.kind = kind
        self.action = action
    }
    func perform() async throws -> some IntentResult {
        NshUIStore.apply(kind: kind, action: action)
        return .result()
    }
}

extension Color {
    init(hex: String) {
        var v: UInt64 = 0
        Scanner(string: hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))).scanHexInt64(&v)
        self.init(red: Double((v >> 16) & 0xff) / 255, green: Double((v >> 8) & 0xff) / 255, blue: Double(v & 0xff) / 255)
    }
}

struct NodeView: View {
    let node: UINode
    let data: [String: Any]
    let kind: String
    var font: String? = nil
    var date: Date = Date()
    @Environment(\.colorScheme) private var scheme
    @Environment(\.widgetFamily) private var family

    private var familyName: String {
        switch family { case .systemSmall: "small"; case .systemLarge, .systemExtraLarge: "large"; default: "medium" }
    }
    private var s: UIStyle { node.style ?? UIStyle() }
    private func pick(_ pair: [String]?) -> Color? {
        guard let p = pair, p.count == 2 else { return nil }
        return Color(hex: scheme == .dark ? p[1] : p[0])
    }
    private func fill(_ text: String) -> String {
        var out = text
        for (k, v) in data {
            let str: String
            if let n = v as? NSNumber, CFGetTypeID(n) != CFBooleanGetTypeID() {
                str = n.doubleValue == n.doubleValue.rounded() ? String(n.intValue) : n.stringValue
            } else { str = "\(v)" }
            out = out.replacingOccurrences(of: "{\(k)}", with: str)
        }
        return out
    }
    private var weight: Font.Weight {
        switch s.weight { case "medium": .medium; case "semibold": .semibold; case "bold": .bold; default: .regular }
    }
    private var hAlign: HorizontalAlignment {
        switch s.align { case "center": .center; case "end": .trailing; default: .leading }
    }
    private var vAlign: VerticalAlignment {
        switch s.align { case "start": .top; case "end": .bottom; default: .center }
    }
    private var kids: [UINode] { node.children ?? [] }
    private func number(_ key: String?) -> Double? { key.flatMap { (data[$0] as? NSNumber)?.doubleValue } }
    private var fgColor: Color { pick(s.fg) ?? .primary }
    private static let symbols = ["star": "star.fill", "bolt": "bolt.fill", "check": "checkmark.circle.fill", "play": "play.fill", "flame": "flame.fill"]
    private func styledText(_ t: Text) -> some View {
        t.font(font.map { Font.custom($0, fixedSize: s.size ?? 14).weight(weight) } ?? .system(size: s.size ?? 14, weight: weight))
            .foregroundStyle(fgColor)
            .monospacedDigit()
            .lineLimit(1)
    }

    @ViewBuilder private var label: some View {
        let t = Text(fill(node.text ?? ""))
            .font(font.map { Font.custom($0, fixedSize: s.size ?? 14).weight(weight) } ?? .system(size: s.size ?? 14, weight: weight))
            .foregroundStyle(fgColor)
        if s.animate == "number" {
            t.lineLimit(1).minimumScaleFactor(0.6).contentTransition(.numericText()).animation(.smooth, value: fill(node.text ?? ""))
        } else {
            t.lineLimit(1).minimumScaleFactor(0.6)
        }
    }

    @ViewBuilder private var content: some View {
        switch node.t {
        case "col":
            VStack(alignment: hAlign, spacing: s.gap ?? 0) {
                ForEach(Array(kids.enumerated()), id: \.offset) { NodeView(node: $0.element, data: data, kind: kind, font: font, date: date) }
            }
            .frame(maxWidth: .infinity, maxHeight: s.grow == true ? .infinity : nil, alignment: .topLeading)
        case "row":
            HStack(alignment: vAlign, spacing: s.gap ?? 0) {
                ForEach(Array(kids.enumerated()), id: \.offset) { NodeView(node: $0.element, data: data, kind: kind, font: font, date: date) }
            }
        case "text":
            label
        case "button":
            if let a = node.action, a.hasPrefix("open:"), let url = URL(string: String(a.dropFirst(5))) {
                Link(destination: url) { decorated(label) }.fixedSize()
            } else {
                Button(intent: NshActionIntent(kind: kind, action: node.action ?? "")) { decorated(label) }
                    .buttonStyle(.plain)
                    .fixedSize()
            }
        case "spacer":
            Spacer(minLength: 0)
        case "progress":
            let v = min(max(number(node.key) ?? 0, 0), 1)
            Capsule().fill(pick(s.bg) ?? .secondary.opacity(0.2))
                .overlay(alignment: .leading) {
                    GeometryReader { g in Capsule().fill(fgColor).frame(width: g.size.width * v) }
                }
                .frame(height: s.height ?? 8)
        case "bars":
            let values = (node.key.flatMap { data[$0] as? [NSNumber] } ?? []).map(\.doubleValue)
            let top = max(values.max() ?? 1, 1)
            HStack(alignment: .bottom, spacing: s.gap ?? 4) {
                ForEach(Array(values.enumerated()), id: \.offset) { item in
                    RoundedRectangle(cornerRadius: 3, style: .continuous).fill(fgColor)
                        .frame(maxWidth: .infinity)
                        .frame(height: max(2, (s.height ?? 48) * item.element / top))
                }
            }
            .frame(height: s.height ?? 48, alignment: .bottom)
        case "icon":
            Image(systemName: Self.symbols[node.name ?? ""] ?? "questionmark.circle")
                .font(.system(size: s.height ?? s.size ?? 18, weight: weight))
                .foregroundStyle(fgColor)
        case "timer":
            let at = Date(timeIntervalSince1970: (number(node.key) ?? 0) / 1000)
            if node.mode == "down" {
                styledText(Text(timerInterval: Date()...max(at, Date()), countsDown: true))
            } else {
                styledText(Text(at, style: .timer))
            }
        case "clock":
            styledText(Text(date, style: .time))
        case "toggle":
            let on = (node.key.flatMap { data[$0] as? Bool }) ?? false
            // Widgets draw Toggle only as a tinted pill (forcing .switch shows the unsupported placeholder), and the pill hides
            // a tinted label when on. A checklist row pressed through the same intent reads clearly in both states.
            Button(intent: NshActionIntent(kind: kind, action: node.action ?? "")) {
                HStack(spacing: 8) {
                    Image(systemName: on ? "checkmark.circle.fill" : "circle")
                        .font(.system(size: (s.size ?? 14) + 4, weight: .semibold))
                        .foregroundStyle(on ? fgColor : Color.secondary)
                    styledText(Text(fill(node.text ?? "")))
                        .strikethrough(on, color: .secondary)
                        .opacity(on ? 0.55 : 1)
                    Spacer(minLength: 0)
                }
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
        case "rotate":
            // WidgetKit has no auto-flipping view: the timeline brings one entry per minute, and each shows the next child.
            if !kids.isEmpty {
                let step = max(node.interval ?? 60, 60)
                let i = Int(date.timeIntervalSince1970 / step) % kids.count
                NodeView(node: kids[i], data: data, kind: kind, font: font, date: date)
                    .id(i)
                    .transition(.push(from: .bottom))
            }
        default:
            EmptyView()
        }
    }

    private var backgroundStyle: AnyShapeStyle {
        if let g = s.gradient, let from = pick(g.from), let to = pick(g.to) {
            let (a, b): (UnitPoint, UnitPoint) = g.dir == "r" ? (.leading, .trailing) : g.dir == "br" ? (.topLeading, .bottomTrailing) : (.top, .bottom)
            return AnyShapeStyle(LinearGradient(colors: [from, to], startPoint: a, endPoint: b))
        }
        return AnyShapeStyle(pick(s.bg) ?? .clear)
    }

    private func decorated<V: View>(_ v: V) -> some View {
        let p = s.pad ?? [0, 0, 0, 0]
        let track = node.t == "progress" // a progress bar's bg is its track, drawn inside it
        return v.padding(EdgeInsets(top: p[0], leading: p[3], bottom: p[2], trailing: p[1]))
            .background(track ? AnyShapeStyle(Color.clear) : backgroundStyle,
                        in: RoundedRectangle(cornerRadius: min(s.radius ?? 0, 200), style: .continuous))
    }

    var body: some View {
        if s.hide?.contains(familyName) == true { EmptyView() }
        else if node.t == "button" { content } else { decorated(content) }
    }
}

struct NshEntry: TimelineEntry {
    let date: Date
    let snapshot: UISnapshot?
    let data: [String: Any]
}

struct NshProvider: TimelineProvider {
    var kind: String
    func placeholder(in context: Context) -> NshEntry { NshEntry(date: Date(), snapshot: nil, data: [:]) }
    func getSnapshot(in context: Context, completion: @escaping (NshEntry) -> Void) { completion(entry()) }
    func getTimeline(in context: Context, completion: @escaping (Timeline<NshEntry>) -> Void) {
        // Ten one-minute entries let `rotate` and `clock` move without the app; timers tick on their own.
        let now = Date()
        let entries = (0..<10).map { entry(now.addingTimeInterval(Double($0) * 60)) }
        completion(Timeline(entries: entries, policy: .atEnd))
    }
    private func entry(_ date: Date = Date()) -> NshEntry {
        NshEntry(date: date, snapshot: NshUIStore.snapshot(kind), data: NshUIStore.data(kind))
    }
}

struct NshWidgetView: View {
    let entry: NshEntry
    var kind: String
    @Environment(\.colorScheme) private var scheme
    var body: some View {
        let root = entry.snapshot?.root
        let bg = root?.style?.bg
        Group {
            if let root {
                // The root's own background becomes the widget container background.
                NodeView(node: UINode(t: root.t, style: UIStyle(fg: root.style?.fg, size: root.style?.size, weight: root.style?.weight,
                                                               gap: root.style?.gap, align: root.style?.align, grow: root.style?.grow),
                                      text: root.text, action: root.action, children: root.children),
                         data: entry.data, kind: kind, font: entry.snapshot?.font, date: entry.date)
            } else {
                Text("Open the app to set up this widget").font(.caption)
            }
        }
        .widgetURL(entry.snapshot?.url.flatMap(URL.init(string:)))
        .containerBackground(for: .widget) {
            if let g = root?.style?.gradient {
                let (a, b): (UnitPoint, UnitPoint) = g.dir == "r" ? (.leading, .trailing) : g.dir == "br" ? (.topLeading, .bottomTrailing) : (.top, .bottom)
                LinearGradient(colors: [g.from, g.to].map { Color(hex: scheme == .dark ? $0[1] : $0[0]) }, startPoint: a, endPoint: b)
            } else if let bg, bg.count == 2 { Color(hex: scheme == .dark ? bg[1] : bg[0]) } else { Color.clear }
        }
    }
}

/// Every kind is the same generic widget with its own kind, name and sizes; `nsh-widgets build` writes one per kind.
func nshUIConfiguration(_ kind: String, _ name: String, _ about: String, _ families: [WidgetFamily]) -> some WidgetConfiguration {
    StaticConfiguration(kind: kind, provider: NshProvider(kind: kind)) { NshWidgetView(entry: $0, kind: kind) }
        .configurationDisplayName(name)
        .description(about)
        .supportedFamilies(families)
}

