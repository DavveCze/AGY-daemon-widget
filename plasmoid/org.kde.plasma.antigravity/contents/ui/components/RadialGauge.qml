import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

ColumnLayout {
    id: gaugeRoot

    property real percentage: 0
    property string title: ""
    property string subtitle: ""
    property color gaugeColor: Kirigami.Theme.positiveTextColor

    Layout.fillWidth: true
    Layout.alignment: Qt.AlignHCenter
    spacing: 4

    Canvas {
        id: canvas
        Layout.alignment: Qt.AlignHCenter
        width: 58
        height: 58

        onPaint: {
            var ctx = getContext("2d");
            ctx.reset();
            var cx = width / 2;
            var cy = height / 2;
            var radius = (width / 2) - 5;

            // Background arc (dark gray / muted track)
            ctx.beginPath();
            ctx.arc(cx, cy, radius, 0, 2 * Math.PI, false);
            ctx.lineWidth = 5;
            ctx.strokeStyle = Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.15);
            ctx.stroke();

            // Progress arc
            var pct = Math.min(Math.max(gaugeRoot.percentage, 0), 100);
            var startAngle = -0.5 * Math.PI;
            var endAngle = startAngle + (pct / 100) * 2 * Math.PI;

            ctx.beginPath();
            ctx.arc(cx, cy, radius, startAngle, endAngle, false);
            ctx.lineWidth = 5;
            ctx.lineCap = "round";
            ctx.strokeStyle = gaugeRoot.gaugeColor;
            ctx.stroke();
        }

        Connections {
            target: gaugeRoot
            function onPercentageChanged() {
                canvas.requestPaint();
            }
            function onGaugeColorChanged() {
                canvas.requestPaint();
            }
        }

        Text {
            anchors.centerIn: parent
            text: Math.round(gaugeRoot.percentage) + "%"
            font.bold: true
            font.pixelSize: 12
            color: Kirigami.Theme.textColor
        }
    }

    Text {
        text: gaugeRoot.title
        font.bold: true
        font.pixelSize: 11
        color: Kirigami.Theme.textColor
        Layout.alignment: Qt.AlignHCenter
    }

    Text {
        text: gaugeRoot.subtitle
        font.pixelSize: 9
        color: Kirigami.Theme.disabledTextColor
        Layout.alignment: Qt.AlignHCenter
    }
}
