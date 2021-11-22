<?php

namespace App\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\ValueObject\DiskDetailsValueObject;
use Symfony\Component\Process\Process;

class Disk implements IReader
{
    public function __construct()
    {
    }

    public function read(): \ArrayObject
    {
        $diskDetails = new \ArrayObject();
        $process = Process::fromShellCommandline("df | grep '^/dev/*'");
        $process->run();

        $result = [];
        if ($process->isSuccessful()) {
            $result = $this->parse($process->getOutput());
        }

        /* @phpstan-ignore-next-line */
        if (0 != \count($result)) {
            /* @phpstan-ignore-next-line */
            foreach ($result as $item) {
                $diskDetails->append(
                    new DiskDetailsValueObject(
                        $item['device'],
                        $item['used'] + $item['available'],
                        $item['used'],
                        $item['available'],
                        $item['percentage'],
                        $item['mounted_at']
                    )
                );
            }
        }

        return $diskDetails;
    }

    public function parse(string $fileContents): array|object|string
    {
        $diskDetails = [];

        $list = \array_filter(\explode("\n", $fileContents));
        foreach ($list as $item) {
            $details = \array_values(\array_filter(\explode(' ', $item)));
            $diskDetails[] = [
                'device' => $details[0],
                'used' => $details[2],
                'available' => $details[3],
                'percentage' => $details[4],
                'mounted_at' => $details[5],
            ];
        }

        return $diskDetails;
    }
}
