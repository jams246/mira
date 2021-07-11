<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use App\Ampere\SystemInfo\ValueObject\UptimeValueObject;
use PHPUnit\Framework\TestCase;

class UptimeValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new UptimeValueObject(-1.0, 1.0));
    }

    /**
     * @dataProvider validUptimeValueObjectValueProvider
     */
    public function testGetValues(float $uptime, float $idled, array $expected): void
    {
        $object = new UptimeValueObject($uptime, $idled);

        $this->assertSame($expected[0], $object->getUptime());
        $this->assertSame($expected[1], $object->getIdled());
    }

    public function validUptimeValueObjectValueProvider(): array
    {
        return [
            [
                1.0,
                2.0,
                [1.0, 2.0],
            ],
        ];
    }
}
