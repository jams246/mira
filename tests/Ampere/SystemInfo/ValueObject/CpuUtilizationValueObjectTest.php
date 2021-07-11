<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\CpuUtilizationValueObject;
use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use PHPUnit\Framework\TestCase;

class CpuUtilizationValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new CpuUtilizationValueObject(-1.0, 1.0));
    }

    /**
     * @dataProvider validCpuUtilizationValueProvider
     */
    public function testGetValues(float $utilization, float $timeRecorded, array $expected): void
    {
        $object = new CpuUtilizationValueObject($utilization, $timeRecorded);

        $this->assertSame($expected[0], $object->getUtilization());
        $this->assertSame($expected[1], $object->getTimeRecorded());
    }

    public function validCpuUtilizationValueProvider(): array
    {
        return [
            [
                1.0,
                2.4,
                [1.0, 2.4],
            ],
            [
                1.872305730,
                123242.53344,
                [1.872305730, 123242.53344],
            ],
        ];
    }
}
