<?php

namespace App\Ampere\SystemInfo;

use App\Ampere\SystemInfo\Dto\LiveInfoDto;

class ThreeSeconds extends BaseLive
{
    private const DATE_FORMAT = ['short' => true, 'join' => false, 'parts' => 2];

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
            $formattedProcess->upTime = \Carbon\CarbonInterval::seconds($process->getUpTime())->cascade()->forHumans(
                self::DATE_FORMAT
            );

            $formattedProcessList->append($formattedProcess);
        }
        $liveInfoDto->setProcessList($formattedProcessList);

        $dockerList = (array) $this->docker->read();
        if (0 != \count($dockerList)) {
            $formattedDockerList = new \ArrayObject();
            foreach ($dockerList as $docker) {
                $formattedDocker = new \stdClass();
                $formattedDocker->name = $docker->getName();
                $formattedDocker->state = $docker->getState();
                $formattedDocker->cpu = $docker->getCpu();
                $formattedDocker->memory = \ByteUnits\Binary::bytes($docker->getMemory())->format();
                $formattedDocker->upTime = \Carbon\CarbonInterval::seconds($docker->getUpTime())->cascade()->forHumans(
                    self::DATE_FORMAT
                );

                $formattedDockerList->append($formattedDocker);
            }

            $liveInfoDto->setDockerList($formattedDockerList);
        }

        $diskList = $this->disk->read();
        $formattedDiskList = new \ArrayObject();
        foreach ($diskList as $disk) {
            $formattedDisk = new \stdClass();
            $formattedDisk->device = $disk->getDevice();
            $formattedDisk->total = \ByteUnits\Metric::kilobytes($disk->getTotal())->format();
            $formattedDisk->used = \ByteUnits\Metric::kilobytes($disk->getUsed())->format();
            $formattedDisk->available = \ByteUnits\Metric::kilobytes($disk->getAvailable())->format();
            $formattedDisk->percentageUsed = $disk->getPercentageUsed();

            $formattedDiskList->append($formattedDisk);
        }
        $liveInfoDto->setDiskList($formattedDiskList);

        return $liveInfoDto;
    }
}
