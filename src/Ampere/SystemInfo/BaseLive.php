<?php

namespace App\Ampere\SystemInfo;

use App\Ampere\SystemInfo\Reader\CpuUtilization;
use App\Ampere\SystemInfo\Reader\Disk;
use App\Ampere\SystemInfo\Reader\Docker;
use App\Ampere\SystemInfo\Reader\LoadAverage;
use App\Ampere\SystemInfo\Reader\Memory;
use App\Ampere\SystemInfo\Reader\Processes;
use App\Ampere\SystemInfo\Reader\SwapMemory;
use App\Ampere\SystemInfo\Reader\Uptime;

abstract class BaseLive
{
    public function __construct(
        protected CpuUtilization $cpuUtilization,
        protected Memory $memory,
        protected LoadAverage $loadAverage,
        protected Processes $processes,
        protected SwapMemory $swapMemory,
        protected Uptime $uptime,
        protected Docker $docker,
        protected Disk $disk
    ) {
    }
}
