<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\CpuUtilizationReadtimeValueObject;
use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use PHPUnit\Framework\TestCase;

class CpuUtilizationReadtimeValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new CpuUtilizationReadtimeValueObject(-1.0, 1.0));
    }

    /**
     * @dataProvider validCpuUtilizationReadtimeValueProvider
     */
    public function testGetValues(float $totalTime, float $idleTime, array $expected): void
    {
        $object = new CpuUtilizationReadtimeValueObject($totalTime, $idleTime);

        $this->assertSame($expected[0], $object->getTotalTime());
        $this->assertSame($expected[1], $object->getIdleTime());
    }

    public function validCpuUtilizationReadtimeValueProvider(): array
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
