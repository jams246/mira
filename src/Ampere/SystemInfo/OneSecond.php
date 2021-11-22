<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo;

use App\Ampere\SystemInfo\Dto\LiveInfoDto;

class OneSecond extends BaseLive
{
    private const DATE_FORMAT = ['short' => true, 'join' => true, 'parts' => 3];

    public function getDto(): LiveInfoDto
    {
        $memory = $this->memory->read();
        $formattedMemory = new \stdClass();
        $formattedMemory->total = \ByteUnits\Metric::kilobytes($memory->getTotal())->format();
        $formattedMemory->free = \ByteUnits\Metric::kilobytes($memory->getFree())->format();
        $formattedMemory->available = \ByteUnits\Metric::kilobytes($memory->getAvailable())->format();
        $formattedMemory->buffers = \ByteUnits\Metric::kilobytes($memory->getBuffers())->format();
        $formattedMemory->cached = \ByteUnits\Metric::kilobytes($memory->getCached())->format();
        $formattedMemory->percentageUsed = $memory->getPercentageUsed();
        $formattedMemory->timeRecorded = $memory->getTimeRecorded();

        $swapMemory = $this->swapMemory->read();
        $formattedSwap = new \stdClass();
        $formattedSwap->total = \ByteUnits\Metric::kilobytes($swapMemory->getTotal())->format();
        $formattedSwap->free = \ByteUnits\Metric::kilobytes($swapMemory->getFree())->format();
        $formattedSwap->cached = \ByteUnits\Metric::kilobytes($swapMemory->getCached())->format();

        $uptime = $this->uptime->read();
        $formattedUptime = new \stdClass();
        $formattedUptime->uptime = \Carbon\CarbonInterval::seconds($uptime->getUptime())->cascade()->forHumans(
            self::DATE_FORMAT
        );
        $formattedUptime->idled = \Carbon\CarbonInterval::seconds($uptime->getIdled())->cascade()->forHumans(
            self::DATE_FORMAT
        );

        return (new LiveInfoDto())
            ->setCpuUtilization($this->cpuUtilization->read())
            ->setLoadAverage($this->loadAverage->read())
            ->setMemory($formattedMemory)
            ->setSwapMemory($formattedSwap)
            ->setUptime($formattedUptime)
        ;
    }
}
