import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { LinksModule } from './links/links.module';

@Module({
  imports: [LinksModule, ConfigModule.forRoot()],
  controllers: [],
  providers: [],
})
export class AppModule {}
