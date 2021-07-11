import Chart from "chart.js/auto";

export default class LoadAverage {

    redBar = {
        backgroundColor: 'rgba(255,105,97,1)',
        borderColor: 'rgba(255,105,97,1)'
    }

    yellowBar = {
        backgroundColor: 'rgba(252,255,164, 1)',
        borderColor: 'rgba(252,255,164, 1)'
    }

    greenBar = {
        backgroundColor: 'rgba(80,200,120, 1)',
        borderColor: 'rgba(80,200,120, 1)'
    }

    constructor(numCores) {
        this.loadThreshold = {
            low: Math.ceil(numCores * 0.25),
            high: Math.floor(numCores * 0.75)
        }

        this.chart = new Chart(
            document.getElementById('loadAvgChart').getContext('2d'),
            {
                type: 'bar',
                data: {
                    labels: ['1 Minute', '5 Minutes', '15 Minutes'],
                    datasets: [{
                        barThickness: 10,
                        categoryPercentage: 0.1,
                        data: new Array(3),
                        backgroundColor: new Array(3),
                        borderColor: new Array(3),
                        borderWidth: 1
                    }]
                },
                options: {
                    indexAxis: 'y',
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        y: {
                            beginAtZero: true
                        }
                    },
                    layout: {
                        padding: 0
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
                    }
                }
            });
    }

    addChartData(reading) {
        this.chart.data.datasets[0].data[0] = reading.minute;
        this.setBarColour(reading.minute, 0);

        this.chart.data.datasets[0].data[1] = reading.fiveMinute;
        this.setBarColour(reading.fiveMinute, 1);

        this.chart.data.datasets[0].data[2] = reading.fifteenMinute;
        this.setBarColour(reading.fifteenMinute, 2);

        return this;
    }

    setBarColour(reading, index) {
        if (reading < this.loadThreshold.low) {
            this.chart.data.datasets[0].backgroundColor[index] = this.greenBar.backgroundColor;
            this.chart.data.datasets[0].borderColor[index] = this.greenBar.borderColor;
        } else if (reading >= this.loadThreshold.high) {
            this.chart.data.datasets[0].backgroundColor[index] = this.redBar.backgroundColor;
            this.chart.data.datasets[0].borderColor[index] = this.redBar.borderColor;
        } else {
            this.chart.data.datasets[0].backgroundColor[index] = this.yellowBar.backgroundColor;
            this.chart.data.datasets[0].borderColor[index] = this.yellowBar.borderColor;
        }
    }

    updateChart() {
        this.chart.update();
    }
}