<?php

declare(strict_types=1);

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use App\Ampere\SystemInfo\ValueObject\IPAddressValueObject;
use PHPUnit\Framework\TestCase;

final class IPAddressValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new IPAddressValueObject('will-throw-exception'));
    }

    /**
     * @dataProvider validIPAddressValueProvider
     */
    public function testGetValidIPAddressValue(string $input, string $expected): void
    {
        $this->assertSame(
            $expected,
            (new IPAddressValueObject($input))->getValue()
        );
    }

    public function validIPAddressValueProvider(): array
    {
        return [
            [
                'Unknown', 'Unknown',
            ],
            [
                '192.168.0.1', '192.168.0.1',
            ],
        ];
    }
}
