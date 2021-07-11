<?php

declare(strict_types=1);

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\BitVersionValueObject;
use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use PHPUnit\Framework\TestCase;

final class BitVersionValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new BitVersionValueObject('will-throw-exception'));
    }

    /**
     * @dataProvider validBitVersionValueProvider
     */
    public function testGetValidBitVersionValue(string $input, string $expected): void
    {
        $this->assertSame(
            $expected,
            (new BitVersionValueObject($input))->getValue()
        );
    }

    public function validBitVersionValueProvider(): array
    {
        return [
            [
                'Unknown', 'Unknown',
            ],
            [
                '32bit', '32bit',
            ],
            [
                '64bit', '64bit',
            ],
        ];
    }
}
