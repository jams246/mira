<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use App\Ampere\SystemInfo\ValueObject\ProcessInfoValueObject;
use PHPUnit\Framework\TestCase;

class ProcessInfoValueObjectTest extends TestCase
{
    /**
     * @dataProvider invalidProcessInfoValueProvider
     */
    public function testInvalidValueThrowsValueObjectException(int $pid, string $processName, int $memoryUsage, string $state, float $cpuUsage, float $uptime): void
    {
        $this->expectException(ValueObjectException::class);

        (new ProcessInfoValueObject($pid, $processName, $memoryUsage, $state, $cpuUsage, $uptime));
    }

    /**
     * @dataProvider validProcessInfoValueProvider
     */
    public function testGetValues(int $pid, string $processName, int $memoryUsage, string $state, float $cpuUsage, float $uptime, array $expected): void
    {
        $object = new ProcessInfoValueObject($pid, $processName, $memoryUsage, $state, $cpuUsage, $uptime);

        $this->assertSame($expected[0], $object->getPid());
        $this->assertSame($expected[1], $object->getProcessName());
        $this->assertSame($expected[2], $object->getMemoryUsage());
        $this->assertSame($expected[3], $object->getState());
        $this->assertSame($expected[4], $object->getCpuUsage());
        $this->assertSame($expected[5], $object->getUpTime());
    }

    public function validProcessInfoValueProvider(): array
    {
        return [
            [
                2,
                'Process name',
                1,
                'R (running)',
                1.0,
                1.0,
                [2, 'Process name', 1, 'R (running)', 1.0, 1.0],
            ],
            [
                2,
                'Process name',
                1,
                'S (sleeping)',
                1.0,
                1.0,
                [2, 'Process name', 1, 'S (sleeping)', 1.0, 1.0],
            ],
            [
                2,
                'Process name',
                1,
                'I (idle)',
                1.0,
                1.0,
                [2, 'Process name', 1, 'I (idle)', 1.0, 1.0],
            ],
        ];
    }

    public function invalidProcessInfoValueProvider(): array
    {
        return [
            [
                -1,
                'Process name',
                1,
                'R (running)',
                1.0,
                1.0,
            ],
            [
                1,
                'Process name',
                1,
                'This-is-an-invalid-state',
                1.0,
                1.0,
            ],
        ];
    }
}
