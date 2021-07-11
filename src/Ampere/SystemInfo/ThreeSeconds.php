<?php

namespace App\Ampere\SystemInfo;

use App\Ampere\SystemInfo\Dto\LiveInfoDto;

class ThreeSeconds extends BaseLive
{
    public function getDto(): LiveInfoDto
    {
        $liveInfoDto = new LiveInfoDto();

        $processList = $this->processes->read();
        $formattedProcessList = new \ArrayObject();
        foreach ($processList as $process) {
            $formattedProcess = new \stdClass();
            $formattedProcess->pid = $process->getPid();
            $formattedProcess->processName = $process->getProcessName();
            $formattedProcess->memoryUsage = \ByteUnits\Metric::kilobytes($process->getMemoryUsage())->format();
            $formattedProcess->state = $process->getState();
            $formattedProcess->cpuUsage = $process->getCpuUsage();
            $formattedProcess->upTime = \Carbon\CarbonInterval::seconds($process->getUpTime())->cascade()->forHumans(['short' => true]);

            $formattedProcessList->append($formattedProcess);
        }
        $liveInfoDto->setProcessList($formattedProcessList);

        if (isset($this->docker)) {
            $dockerList = (array) $this->docker->read();
            $formattedDockerList = new \ArrayObject();
            foreach ($dockerList as $docker) {
                $formattedDocker = new \stdClass();
                $formattedDocker->name = $docker->getName();
                $formattedDocker->state = $docker->getState();
                $formattedDocker->cpu = $docker->getCpu();
                $formattedDocker->memory = \ByteUnits\Binary::bytes($docker->getMemory())->format();

                $formattedDockerList->append($formattedDocker);
            }

            $liveInfoDto->setDockerList($formattedDockerList);
        }

        return $liveInfoDto;
    }
}
