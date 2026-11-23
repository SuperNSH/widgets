// The nsh-widgets renderer for KDE Plasma: draws one Widget Tree IR v1 node and, through Loaders of
// this same file, its children. It knows nothing about any one widget, like the SwiftUI and RemoteViews
// renderers; `run` is how a pressed button reaches `nsh-widgets apply`.
import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: node
    property var n: ({})
    property var vals: ({})
    property bool dark: false
    property real now: 0
    property string family: "medium"
    property bool inRow: false
    property string fontFamily: ""
    property var run: function (action) {}

    readonly property var st: (n && n.style) ? n.style : ({})
    readonly property var pad: st.pad || [0, 0, 0, 0]
    readonly property bool hidden: !!(st.hide && st.hide.indexOf(family) >= 0)
    readonly property color fg: pick(st.fg) || (dark ? "#ffffff" : "#111111")
    readonly property int weight: st.weight === "bold" ? Font.Bold : st.weight === "semibold" ? Font.DemiBold
        : st.weight === "medium" ? Font.Medium : Font.Normal

    visible: !hidden
    implicitWidth: hidden || !body.item ? 0 : body.item.implicitWidth
    implicitHeight: hidden || !body.item ? 0 : body.item.implicitHeight
    // A row with a spacer stretches, as SwiftUI's HStack with a Spacer does.
    readonly property bool hasSpacer: !!(n.children && n.children.some(function (c) { return c.t === "spacer" }))
    Layout.fillWidth: !hidden && (n.t === "col" || (n.t === "row" && hasSpacer) || n.t === "progress" || n.t === "bars" || n.t === "list"
        || n.t === "rotate" || (n.t === "spacer" && inRow) || (n.t === "toggle"))
    Layout.fillHeight: !hidden && ((n.t === "spacer" && !inRow) || st.grow === true || n.t === "list")
    Layout.preferredWidth: implicitWidth
    Layout.preferredHeight: implicitHeight
    Layout.alignment: inRow ? Qt.AlignVCenter : Qt.AlignLeft

    function pick(p) { return p && p.length === 2 ? (dark ? p[1] : p[0]) : "" }
    function fill(t, v) {
        var d = v || vals
        return (t || "").replace(/\{([A-Za-z_][\w.]*)\}/g, function (m, k) {
            var x = d[k]
            if (x === undefined) return m
            return typeof x === "number" && Math.round(x) === x ? String(Math.round(x)) : String(x)
        })
    }
    function dur(ms) {
        var s = Math.max(0, Math.floor(ms / 1000)), h = Math.floor(s / 3600), m = Math.floor(s / 60) % 60, x = s % 60
        function p(v) { return v < 10 ? "0" + v : "" + v }
        return h > 0 ? h + ":" + p(m) + ":" + p(x) : m + ":" + p(x)
    }
    function glyph(name) {
        return ({ star: "★", bolt: "⚡", check: "✓", play: "▶", pause: "⏸", flame: "🔥", plus: "+", minus: "−",
            "arrow-up": "↑", "arrow-down": "↓", heart: "♥", home: "⌂", person: "👤", bell: "🔔", clock: "🕒",
            calendar: "📅", cloud: "☁", sun: "☀", moon: "☾", drop: "💧", music: "♪" })[name] || "?"
    }
    function num(k) { var x = vals[k]; return typeof x === "number" ? x : 0 }
    function childProps(c, v, row) {
        return { n: c, vals: v || vals, dark: dark, now: now, family: family, inRow: row, fontFamily: fontFamily, run: run }
    }

    Loader {
        id: body
        active: !node.hidden
        anchors.fill: parent
        sourceComponent: ({ col: box, row: box, text: label, button: button, spacer: spacer, progress: progress,
            bars: bars, icon: icon, timer: label, clock: label, toggle: toggle, rotate: rotate, list: list })[node.n.t] || spacer
    }

    Component {
        id: box
        Rectangle {
            radius: Math.min(node.st.radius || 0, height / 2)
            color: node.st.gradient ? "transparent" : (node.pick(node.st.bg) || "transparent")
            Rectangle {
                visible: !!node.st.gradient
                anchors.fill: parent
                radius: parent.radius
                gradient: Gradient {
                    orientation: node.st.gradient && node.st.gradient.dir === "r" ? Gradient.Horizontal : Gradient.Vertical
                    GradientStop { position: 0; color: node.st.gradient ? node.pick(node.st.gradient.from) : "transparent" }
                    GradientStop { position: 1; color: node.st.gradient ? node.pick(node.st.gradient.to) : "transparent" }
                }
            }
            implicitWidth: lay.implicitWidth + node.pad[1] + node.pad[3]
            implicitHeight: lay.implicitHeight + node.pad[0] + node.pad[2]
            GridLayout {
                id: lay
                anchors.fill: parent
                anchors.topMargin: node.pad[0]; anchors.rightMargin: node.pad[1]
                anchors.bottomMargin: node.pad[2]; anchors.leftMargin: node.pad[3]
                flow: node.n.t === "row" ? GridLayout.LeftToRight : GridLayout.TopToBottom
                rows: node.n.t === "row" ? 1 : -1
                columns: node.n.t === "row" ? -1 : 1
                rowSpacing: node.st.gap || 0
                columnSpacing: node.st.gap || 0
                Repeater {
                    model: node.n.children || []
                    delegate: Loader {
                        Layout.fillWidth: item ? item.Layout.fillWidth : false
                        Layout.fillHeight: item ? item.Layout.fillHeight : false
                        Layout.preferredWidth: item ? item.implicitWidth : 0
                        Layout.preferredHeight: item ? item.implicitHeight : 0
                        Layout.alignment: node.n.t === "row" ? (Qt.AlignVCenter | Qt.AlignLeft)
                            : node.st.align === "center" ? Qt.AlignHCenter : node.st.align === "end" ? Qt.AlignRight : Qt.AlignLeft
                        Component.onCompleted: setSource("NshNode.qml", node.childProps(modelData, null, node.n.t === "row"))
                        onLoaded: item.now = Qt.binding(function () { return node.now })
                    }
                }
                // A row without a spacer keeps its children at the start when it is wider than they are.
                Item {
                    visible: node.n.t === "row" && !node.hasSpacer
                    Layout.fillWidth: visible
                    Layout.preferredWidth: 0
                }
            }
        }
    }

    Component {
        id: label
        Text {
            text: node.n.t === "timer"
                ? node.dur(node.n.mode === "down" ? node.num(node.n.key) - node.now : node.now - node.num(node.n.key))
                : node.n.t === "clock" ? Qt.formatTime(new Date(node.now), "hh:mm") : node.fill(node.n.text)
            color: node.fg
            font.pixelSize: node.st.size || 14
            font.weight: node.weight
            font.family: node.fontFamily || font.family
        }
    }

    Component {
        id: button
        Rectangle {
            radius: Math.min(node.st.radius || 0, height / 2)
            color: node.pick(node.st.bg) || "transparent"
            implicitWidth: t.implicitWidth + node.pad[1] + node.pad[3]
            implicitHeight: t.implicitHeight + node.pad[0] + node.pad[2]
            Text {
                id: t
                anchors.centerIn: parent
                text: node.fill(node.n.text)
                color: node.fg
                font.pixelSize: node.st.size || 14
                font.weight: node.weight
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: node.run(node.n.action || "") }
        }
    }

    Component { id: spacer; Item { implicitWidth: 0; implicitHeight: 0 } }

    Component {
        id: progress
        Rectangle {
            implicitHeight: node.st.height || 8
            implicitWidth: 40
            radius: height / 2
            color: node.pick(node.st.bg) || Qt.rgba(0.5, 0.5, 0.5, 0.25)
            Rectangle {
                width: parent.width * Math.max(0, Math.min(1, node.num(node.n.key)))
                height: parent.height
                radius: height / 2
                color: node.fg
            }
        }
    }

    Component {
        id: bars
        Item {
            id: r
            readonly property var values: (node.vals[node.n.key] || []).filter(function (v) { return typeof v === "number" })
            readonly property real peak: Math.max.apply(null, [1].concat(values))
            readonly property real chartHeight: node.st.height || 48
            readonly property real gap: node.st.gap || 4
            implicitHeight: chartHeight
            implicitWidth: 40
            Row {
                anchors.fill: parent
                spacing: r.gap
                Repeater {
                    model: r.values
                    delegate: Item {
                        width: (r.width - r.gap * Math.max(0, r.values.length - 1)) / Math.max(1, r.values.length)
                        height: r.chartHeight
                        Rectangle {
                            anchors.bottom: parent.bottom
                            width: parent.width
                            height: Math.max(2, r.chartHeight * modelData / r.peak)
                            radius: 3
                            color: node.fg
                        }
                    }
                }
            }
        }
    }

    Component {
        id: icon
        Text {
            text: node.glyph(node.n.name)
            color: node.fg
            font.pixelSize: node.st.height || node.st.size || 18
        }
    }

    Component {
        id: toggle
        Item {
            readonly property bool on: node.vals[node.n.key] === true
            implicitHeight: Math.max(m.implicitHeight, l.implicitHeight) + 6
            implicitWidth: m.implicitWidth + l.implicitWidth + 8
            Text { id: m; anchors.verticalCenter: parent.verticalCenter; text: parent.on ? "☑" : "☐"
                color: node.fg; font.pixelSize: (node.st.size || 14) + 4 }
            Text { id: l; anchors.left: m.right; anchors.leftMargin: 8; anchors.verticalCenter: parent.verticalCenter
                text: node.fill(node.n.text); color: node.fg; opacity: parent.on ? 0.55 : 1
                font.pixelSize: node.st.size || 14; font.strikeout: parent.on }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: node.run(node.n.action || ("toggle:" + node.n.key)) }
        }
    }

    Component {
        id: rotate
        Loader {
            readonly property var kids: node.n.children || []
            readonly property int index: kids.length ? Math.floor(node.now / (1000 * Math.max(1, node.n.interval || 60))) % kids.length : -1
            onIndexChanged: if (index >= 0) setSource("NshNode.qml", node.childProps(kids[index], null, false))
            Component.onCompleted: if (index >= 0) setSource("NshNode.qml", node.childProps(kids[index], null, false))
        }
    }

    Component {
        id: list
        ListView {
            // Plasma scrolls the list, as Android does, through the first `max` items.
            clip: true
            spacing: node.st.gap || 0
            implicitHeight: Math.min(contentHeight, 400)
            implicitWidth: 40
            model: (node.vals[node.n.key] || []).slice(0, node.n.max || 8)
            delegate: Loader {
                width: ListView.view ? ListView.view.width : 0
                height: item ? item.implicitHeight : 0
                Component.onCompleted: {
                    var v = Object.assign({}, node.vals)
                    for (var k in modelData) v["item." + k] = modelData[k]
                    setSource("NshNode.qml", node.childProps((node.n.children || [])[0] || {}, v, false))
                }
            }
        }
    }
}
