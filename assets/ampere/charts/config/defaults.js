export var lineChart = {
    responsive: true,
    maintainAspectRatio: false,
    layout: {
        padding: 0
    },
    elements: {
        line: {
            tension: 0.4
        }
    },
    plugins: {
        title: {
            display: false
        },
        legend: {
            display: false
        },
        tooltip: {
            enabled: false
        }
    },
    scales: {
        x: {
            grid: {
                display: false
            }
        },
        y: {
            beginAtZero: true,
            min: 0,
            max: 100,
            grid: {
                display: false
            }
        }
    }
}