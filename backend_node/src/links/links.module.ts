import { Module } from '@nestjs/common';
import { LinksController } from './links.controller';
import LinksDbService from './links.db.service';
import { LinksService } from './links.service';
import { MediafilesModule } from 'src/mediafiles/mediafiles.module';

@Module({
  imports: [MediafilesModule],
  controllers: [LinksController],
  providers: [LinksService, LinksDbService],
})
export class LinksModule {}
