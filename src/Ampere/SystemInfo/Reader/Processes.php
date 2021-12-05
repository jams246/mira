<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\ProcessInfoValueObject;
use Psr\Cache\CacheItemPoolInterface;
use Symfony\Component\Finder\Finder;
use Symfony\Component\Finder\SplFileInfo;
use Symfony\Component\Process\Process;

/**
 * @reference https://man7.org/linux/man-pages/man5/procfs.5.html
 */
class Processes implements IReader
{
    private int $clockTicks = 100;
    private int $cpuCoreCount;

    private const PROC_PATH = '/proc';

    private array $groupCount = ['R (running)' => 0, 'S (sleeping)' => 0, 'I (idle)' => 0, 'D (disk sleep)' => 0];

    public function __construct(private CacheItemPoolInterface $cache, private CpuCoreCount $coreCountReader)
    {
        $process = new Process(['getconf', 'CLK_TCK']);
        $process->run();
        if ($process->isSuccessful()) {
            $this->clockTicks = (int) $process->getOutput();
        }

        $this->cpuCoreCount = $this->coreCountReader->read()->getValue();
    }

    public function read(): \ArrayObject
    {
        $finder = new Finder();
        $finder
            ->depth('== 0')
            ->ignoreUnreadableDirs()
            ->directories();

        $processInfo = new \ArrayObject();
        foreach ($finder->in(self::PROC_PATH)->name('/[0-9]+/') as $directory) {
            $statusPath = $directory->getRealPath().'/status';
            $fileInfo = new SplFileInfo($statusPath, $directory->getRelativePath(), $directory->getRelativePathname());
            if (!$fileInfo->isFile()) {
                continue;
            }

            $statusInfo = $this->parse($fileInfo->getContents());

            $state = $statusInfo->State;
            if (\in_array($state, ['Z (zombie)', 'D (disk sleep)'])) {
                continue;
            }

            $processName = $statusInfo->Name;
            if ('php' === $processName) {
                $cmdLine = \file_get_contents($directory->getRealPath().'/cmdline');
                if (false != $cmdLine && \strpos($cmdLine, 'Mira') > -1) {
                    $processName = 'Mira';
                }
            }

            $pid = (int) $statusInfo->Pid;

            try {
                $processStats = $this->calculateProcessStats($pid, $directory);
                $currentMemoryUsage = $this->readMemoryStats($directory);
            } catch (\Exception $e) {
                continue;
            }

            $processInfo->append(new ProcessInfoValueObject(
                $pid,
                $processName,
                $currentMemoryUsage,
                $state,
                $processStats->usage, /* @phpstan-ignore-line */
                /* @phpstan-ignore-next-line */
                $processStats->uptime
            ));
        }

        $processInfo->uasort(function ($a, $b) {
            return $b->getCpuUsage() - $a->getCpuUsage();
        });

        $temp = $processInfo->getArrayCopy();
        \array_splice($temp, 20);

        $order = ['R (running)', 'S (sleeping)', 'I (idle)', 'D (disk sleep)'];
        \uasort($temp, function ($a, $b) use ($order) {
            $posA = \array_search($a->getState(), $order);
            $posB = \array_search($b->getState(), $order);

            return $posA - $posB;
        });

        $temp = \array_map(function ($item) {
            $state = $item->getState();
            if (0 === $this->groupCount[$state]) {
                ++$this->groupCount[$state];

                return $item;
            }

            $item->setState('');

            return $item;
        }, $temp);

        $processInfo->exchangeArray($temp);
        unset($temp);
        $this->groupCount = ['R (running)' => 0, 'S (sleeping)' => 0, 'I (idle)' => 0, 'D (disk sleep)' => 0];

        return $processInfo;
    }

    /**
     * @url https://man7.org/linux/man-pages/man5/proc.5.html #search for /proc/[pid]/stat
     * @url https://stackoverflow.com/a/16736599
     */
    private function calculateProcessStats(int &$pid, SplFileInfo &$directory): object
    {
        $processStats = $this->cache->getItem("process.{$pid}.utilization");
        $stats = [0.0, 0.0];
        if ($processStats->isHit()) {
            $stats = $processStats->get();
        }

        $uptime = (new Uptime())->read()->getUptime();

        $fileObject = new \SplFileObject('/proc/stat');
        $fileContents = $fileObject->current();
        $fileContents = \str_replace(['cpu  ', "\n"], '', $fileContents);
        $totalCpuUsage = \array_values(\array_filter(\explode(' ', $fileContents)));
        $totalCpuUsage = \array_map('floatval', $totalCpuUsage);
        $totalCpuUsage = \array_sum($totalCpuUsage);

        $data = \file_get_contents($directory->getRealPath().'/stat');
        $data = \explode(' ', $data);
        $totalProcessTime = (float) $data[13] + (float) $data[14];

        $cpuUsage = $this->cpuCoreCount * ($totalProcessTime - $stats[1]) * 100 / (float) ($totalCpuUsage - $stats[0]);
        $cpuUsage = \round($cpuUsage, 1);
        /*
         * The cpuUsage value can be under 0 when trying to access the server
         * within one second of it starting.
         */
        if ($cpuUsage <= 0.0) {
            $cpuUsage = 0.1;
        }

        $processStats->set([$totalCpuUsage, $totalProcessTime]);
        $this->cache->save($processStats);

        return (object) [
            'usage' => $cpuUsage,
            'uptime' => $uptime - ($data[21] / $this->clockTicks),
        ];
    }

    private function readMemoryStats(SplFileInfo &$directory): int
    {
        $data = \file_get_contents($directory->getRealPath().'/statm');
        $data = \explode(' ', $data);

        return (int) $data[0];
    }

    public function parse(string $fileContents): array|object|string
    {
        \preg_match_all('/^([^:]+)\:\s+(.*)/m', $fileContents, $matches);

        return (object) \array_combine($matches[1], $matches[2]);
    }
}
