import Chart from "chart.js/auto";
import {lineChart} from "./config/defaults";

export default class UsageChart {
    constructor(element) {
        this.chart = new Chart(element, {
            type: 'line',
            data: {
                labels: new Array(120),
                datasets: [
                    {
                        label: '',
                        data: new Array(120),
                        fill: true,
                        backgroundColor: 'rgba(245,245,245, 0.2)',
                        borderColor: 'rgba(245,245,245, 1)',
                        borderWidth: 1,
                        pointRadius: 0
                    }
                ]
            },
            options: lineChart
        });
    }

    addChartData(reading) {
        this.chart.data.labels.shift();
        this.chart.data.datasets[0].data.shift();
        this.chart.data.labels.push('');
        this.chart.data.datasets[0].data.push(reading);

        return this;
    }

    updateChart() {
        this.chart.update();
    }
}