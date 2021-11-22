import tooltip from 'tooltip';
import {Notyf} from "notyf";
import 'notyf/notyf.min.css';

/** Mira **/
import './styles/app.scss';

import UsageChart from './ampere/charts/UsageChart';
import LoadAverage from "./ampere/charts/LoadAverage";
import ListColumn from "./ampere/functions/ListColumn";
import LiveInfo from "./ampere/functions/LiveInfo";

(function () {
    const loadAvgChart = new LoadAverage(initialReadings.coreCount);
    loadAvgChart.addChartData(initialReadings.loadAvgReadings).updateChart();
    initialReadings = null;

    let dockerListElement = document.getElementById('dockerList') || null;

    new ListColumn('#processList')
        .attachListener(document.getElementById('processListColumnSelect'));

    if (dockerListElement) {
        new ListColumn('#dockerList')
            .attachListener(document.getElementById('dockerListColumnSelect'));
    }

    const cpuUtilizationChart = new UsageChart(document.getElementById('cpuUsageChart').getContext('2d'));
    const memUsageChart = new UsageChart(document.getElementById('memUsageChart').getContext('2d'));
    const notyf = new Notyf({duration:3500,position: {x: 'center', y: 'top'}});
    new LiveInfo(
        document.getElementById('uptime-liveInfo'),
        document.getElementById('memory-liveInfo'),
        document.getElementById('processList'),
        dockerListElement,
        document.getElementById('mobile_CpuMemUsageList'),
        document.getElementById('diskList'),
        cpuUtilizationChart,
        memUsageChart,
        loadAvgChart,
        notyf
    ).useWebsocket();

    tooltip({showDelay: 0});
})();