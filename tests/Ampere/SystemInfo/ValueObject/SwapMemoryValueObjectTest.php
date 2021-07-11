<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use App\Ampere\SystemInfo\ValueObject\SwapMemoryValueObject;
use PHPUnit\Framework\TestCase;

class SwapMemoryValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new SwapMemoryValueObject(-1, 1, 123));
    }

    /**
     * @dataProvider validSwapMemoryValueObjectValueProvider
     */
    public function testGetValues(int $total, int $free, int $cached, array $expected): void
    {
        $object = new SwapMemoryValueObject($total, $free, $cached);

        $this->assertSame($expected[0], $object->getTotal());
        $this->assertSame($expected[1], $object->getFree());
        $this->assertSame($expected[2], $object->getCached());
    }

    public function validSwapMemoryValueObjectValueProvider(): array
    {
        return [
            [
                1,
                2,
                3,
                [1, 2, 3],
            ],
        ];
    }
}
