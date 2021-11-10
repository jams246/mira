<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\CpuUtilizationReadtimeValueObject;
use App\Ampere\SystemInfo\ValueObject\CpuUtilizationValueObject;
use Symfony\Component\Cache\Adapter\AdapterInterface;
use Symfony\Component\Serializer\SerializerInterface;

class CpuUtilization implements IReader
{
    private const STAT_PATH = '/proc/stat';

    private CpuUtilizationReadtimeValueObject $readTimes;

    /**
     * CpuUtilization constructor.
     */
    public function __construct(private AdapterInterface $cache, private SerializerInterface $serializer)
    {
        $this->readTimes = new CpuUtilizationReadtimeValueObject(0.0, 0.0);
    }

    public function read(): CpuUtilizationValueObject
    {
        $cpuReadings = $this->cache->getItem('cpu.utilization');
        if ($cpuReadings->isHit()) {
            /* @phpstan-ignore-next-line */
            $this->readTimes = $this->serializer->deserialize(
                $cpuReadings->get(),
                CpuUtilizationReadtimeValueObject::class,
                'json'
            );
        }

        $fileObject = new \SplFileObject(self::STAT_PATH);
        /* @phpstan-ignore-next-line */
        $allTimes = $this->parse($fileObject->current());

        //Convert all values to float to align with the formula expectations
        $allTimes = \array_map('floatval', $allTimes[0]);

        $cpuUtilization = $this->calculateCpuUtilization($allTimes);

        $timeRecorded = \microtime(true) * 1000;

        $this->cacheReadTimes();

        return new CpuUtilizationValueObject($cpuUtilization, $timeRecorded);
    }

    public function parse(string $fileContents): array
    {
        \preg_match_all('/\d+/', $fileContents, $allTimes);

        return $allTimes;
    }

    /**
     * Formula for calculating CPU utilization from /proc/stat.
     *
     * @url https://rosettacode.org/wiki/Linux_CPU_utilization
     * @url https://rosettacode.org/wiki/Linux_CPU_utilization#Python
     */
    public function calculateCpuUtilization(array $allTimes): float
    {
        //sum all of the times found on that first line to get the total time
        $totalTime = \array_sum($allTimes);

        $totalDelta = $totalTime - $this->readTimes->getTotalTime();
        $this->readTimes->setTotalTime($totalTime);

        //divide the fourth column ("idle") by the total time, to get the fraction of time spent being idle
        $idleTime = $allTimes[3];
        $idleDelta = $idleTime - $this->readTimes->getIdleTime();
        $this->readTimes->setIdleTime($idleTime);

        /*
         * subtract the previous fraction from 1.0 to get the time spent being not idle
         * multiply by 100 to get a percentage.
         */
        return \ceil(100 * (1.0 - ($idleDelta / $totalDelta)));
    }

    private function cacheReadTimes(): void
    {
        $cpuReadings = $this->cache->getItem('cpu.utilization');
        $cpuReadings->set($this->serializer->serialize($this->readTimes, 'json'));

        $this->cache->save($cpuReadings);
    }
}
