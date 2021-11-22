import uptimeTmpl from "../templates/uptime.njk";
import memoryTmpl from "../templates/memory.njk";
import mobile_CpuMemUsageTmpl from "../templates/mobileCpuMemUsage.njk";
import processListTmpl from "../templates/processList.njk";
import dockerListTmpl from "../templates/dockerList.njk";
import ListColumn from "./ListColumn";
import diskListTmpl from "../templates/diskList.njk";

export default class LiveInfo {
    wsReconnectAttempts = 0;
    fetchRequestAttempts = 0;

    constructor(uptimeLiveInfo, memoryLiveInfo, processListInfo, dockerListInfo, mobile_CpuMemUsage, distListInfo, cpuUtilizationChart, memUsageChart, loadAvgChart, notyf) {
        this.uptimeLiveInfo = uptimeLiveInfo;
        this.memoryLiveInfo = memoryLiveInfo;
        this.processListInfo = processListInfo;
        this.dockerListInfo = dockerListInfo;
        this.mobile_CpuMemUsage = mobile_CpuMemUsage;
        this.diskListInfo = distListInfo;
        this.cpuUtilizationChart = cpuUtilizationChart;
        this.memUsageChart = memUsageChart;
        this.loadAvgChart = loadAvgChart;
        this.notyf = notyf;
    }


    useWebsocket() {
        const hostname = window.location.hostname;
        let socket = new WebSocket('ws://' + hostname + ':44357/live.info');
        const self = this;

        socket.onopen = function () {
            if (self.wsReconnectAttempts >= 1) {
                self.notyf.success('Websocket connection reestablished.');
                self.wsReconnectAttempts = 0;
            }
        };
        socket.onmessage = function (event) {
            let liveData = JSON.parse(event.data);

            switch (liveData.event) {
                case "OneSecond":
                    self.updateOneSecond(
                        liveData.data.cpuUtilization.utilization,
                        liveData.data.memory.percentageUsed,
                        liveData.data.loadAverage,
                        liveData.data.uptime.uptime,
                        liveData.data.memory
                    )
                    break;

                case "ThreeSeconds":
                    self.updateThreeSeconds(
                        liveData.data.processList,
                        liveData.data.dockerList,
                        liveData.data.diskList
                    );
                    break;
            }
        };
        socket.onclose = function () {
            if (self.wsReconnectAttempts === 0) {
                self.notyf.error('Websocket connection lost.<br/>Attempting to reconnect.');
            } else {
                self.notyf.error('Retrying.');
            }
            setTimeout(function () {
                ++self.wsReconnectAttempts;
                if (self.wsReconnectAttempts < 5) {
                    self.useWebsocket();
                }
                if (self.wsReconnectAttempts === 5) {
                    self.notyf.error('Falling back to HttpRequests.');
                    self.useHttpRequests();
                }
            }, 5000);
        };
    }

    useHttpRequests() {
        const self = this;

        this.oneSecond = setInterval(function () {
            fetch('/one-second')
                .then(response => response.json())
                .then(data => self.updateOneSecond(
                    data.cpuUtilization.utilization,
                    data.memory.percentageUsed,
                    data.loadAverage,
                    data.uptime.uptime,
                    data.memory
                    )
                ).catch(function () {
                self.handleFetchErrors()
            });
        }, 1000);

        this.threeSecond = setInterval(function () {
            fetch('/three-second')
                .then(response => response.json())
                .then(data => self.updateThreeSeconds(data.processList, data.dockerList, data.diskList)
                ).catch(function () {
                    self.handleFetchErrors()
            });
        }, 3000);
    }

    handleFetchErrors() {
        ++this.fetchRequestAttempts;
        if (this.fetchRequestAttempts > 5) {
            clearInterval(this.oneSecond);
            clearInterval(this.threeSecond);
            this.notyf.error('Unable to establish Websocket or HttpRequest connection.');
        }
    }

    updateOneSecond(cpuUtilization, memoryPercentage, loadAverage, uptime, memoryStats) {
        this.cpuUtilizationChart.addChartData(cpuUtilization).updateChart();
        this.memUsageChart.addChartData(memoryPercentage).updateChart();
        this.loadAvgChart.addChartData(loadAverage).updateChart();

        this.uptimeLiveInfo.innerHTML = uptimeTmpl({uptime: uptime});
        this.memoryLiveInfo.innerHTML = memoryTmpl({memory: memoryStats});
        this.mobile_CpuMemUsage.innerHTML = mobile_CpuMemUsageTmpl({cpu: cpuUtilization, mem: memoryPercentage})
    }

    updateThreeSeconds(processList, dockerList, diskList) {
        this.processListInfo.innerHTML = processListTmpl({processes: processList});
        let selectedValue = document.getElementById('processListColumnSelect').value;
        new ListColumn('#processList').updateDataTable(selectedValue);

        this.dockerListInfo.innerHTML = dockerListTmpl({dockerList: dockerList});
        selectedValue = document.getElementById('dockerListColumnSelect').value;
        new ListColumn('#dockerList').updateDataTable(selectedValue);

        this.diskListInfo.innerHTML = diskListTmpl({diskList: diskList});
    }

}