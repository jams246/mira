<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use App\Ampere\SystemInfo\ValueObject\LoadAverageValueObject;
use PHPUnit\Framework\TestCase;

class LoadAverageValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new LoadAverageValueObject(-1.0, 1.0, 123123.53));
    }

    /**
     * @dataProvider validLoadAverageValueObjectValueProvider
     */
    public function testGetValues(float $oneMinute, float $fiveMinute, float $fifteenMinute, array $expected): void
    {
        $object = new LoadAverageValueObject($oneMinute, $fiveMinute, $fifteenMinute);

        $this->assertSame($expected[0], $object->getMinute());
        $this->assertSame($expected[1], $object->getFiveMinute());
        $this->assertSame($expected[2], $object->getFifteenMinute());
    }

    public function validLoadAverageValueObjectValueProvider(): array
    {
        return [
            [
                1.0,
                2.3,
                4.6,
                [1.0, 2.3, 4.6],
            ],
            [
                1.872305730,
                123242.53344,
                0.1,
                [1.872305730, 123242.53344, 0.1],
            ],
        ];
    }
}
