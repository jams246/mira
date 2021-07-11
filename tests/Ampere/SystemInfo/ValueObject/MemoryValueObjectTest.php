<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use App\Ampere\SystemInfo\ValueObject\MemoryValueObject;
use PHPUnit\Framework\TestCase;

class MemoryValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new MemoryValueObject(23, 23, 23, 23, 23, 23, -1.0));
    }

    /**
     * @dataProvider validMemoryValueObjectValueProvider
     */
    public function testGetValues(int $total, int $free, int $available, int $buffers, int $cached, float $percentageUsed, float $timeRecorded, array $expected): void
    {
        $object = new MemoryValueObject($total, $free, $available, $buffers, $cached, $percentageUsed, $timeRecorded);

        $this->assertSame($expected[0], $object->getTotal());
        $this->assertSame($expected[1], $object->getFree());
        $this->assertSame($expected[2], $object->getAvailable());
        $this->assertSame($expected[3], $object->getBuffers());
        $this->assertSame($expected[4], $object->getCached());
        $this->assertSame($expected[5], $object->getPercentageUsed());
        $this->assertSame($expected[6], $object->getTimeRecorded());
    }

    public function validMemoryValueObjectValueProvider(): array
    {
        return [
            [
                23, 23, 23, 23, 23, 23.0, 23.0,
                [23, 23, 23, 23, 23, 23.0, 23.0],
            ],
        ];
    }
}
